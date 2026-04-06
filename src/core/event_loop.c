/**
 * AlloyScript — Event Loop Implementation
 *
 * Background I/O thread model:
 *
 *   JS thread                     I/O thread
 *   ─────────────────────────     ──────────────────────────────────
 *   alloy_loop_dispatch(payload)  waits on task_cv
 *   → enqueues AlloyTask          ← wakes, dequeues task
 *   → returns task_id             → alloy_ipc_send(payload)
 *   → parks coroutine             → alloy_ipc_recv() [blocking]
 *   → runs other coroutines       → enqueues AlloyCompletion
 *   ← alloy_loop_drain()          → signals comp_cv
 *   → resumes coroutine(result)
 *
 * The JS thread never calls read() directly. It only blocks on comp_cv
 * when the completion queue is empty AND there are pending coroutines.
 */
#include "../../include/alloy/event_loop.h"
#include "../../include/alloy/ipc_proto.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <time.h>
#include <errno.h>

/* ── Queue helpers (caller holds the relevant mutex) ────────────────────── */

static bool task_enqueue(AlloyEventLoop *loop, const AlloyTask *t) {
    if (loop->task_count >= ALLOY_LOOP_QUEUE_CAP) return false;
    loop->task_queue[loop->task_tail] = *t;
    loop->task_tail = (loop->task_tail + 1) % ALLOY_LOOP_QUEUE_CAP;
    loop->task_count++;
    return true;
}

static bool task_dequeue(AlloyEventLoop *loop, AlloyTask *out) {
    if (loop->task_count == 0) return false;
    *out = loop->task_queue[loop->task_head];
    loop->task_head = (loop->task_head + 1) % ALLOY_LOOP_QUEUE_CAP;
    loop->task_count--;
    return true;
}

static bool comp_enqueue(AlloyEventLoop *loop, const AlloyCompletion *c) {
    if (loop->comp_count >= ALLOY_LOOP_QUEUE_CAP) return false;
    loop->comp_queue[loop->comp_tail] = *c;
    loop->comp_tail = (loop->comp_tail + 1) % ALLOY_LOOP_QUEUE_CAP;
    loop->comp_count++;
    return true;
}

static bool comp_dequeue(AlloyEventLoop *loop, AlloyCompletion *out) {
    if (loop->comp_count == 0) return false;
    *out = loop->comp_queue[loop->comp_head];
    loop->comp_head = (loop->comp_head + 1) % ALLOY_LOOP_QUEUE_CAP;
    loop->comp_count--;
    return true;
}

/* ── I/O thread ─────────────────────────────────────────────────────────── */

static void post_completion(AlloyEventLoop *loop, uint32_t task_id,
                             AlloyCompletionType type, char *result) {
    AlloyCompletion c = { .type = type, .task_id = task_id, .result = result };
    pthread_mutex_lock(&loop->comp_mu);
    if (!comp_enqueue(loop, &c)) {
        /* Queue full — this is a bug; log and drop */
        fprintf(stderr, "[alloy:loop] completion queue full, dropping task %u\n", task_id);
        free(result);
    } else {
        pthread_cond_signal(&loop->comp_cv);
    }
    pthread_mutex_unlock(&loop->comp_mu);
}

static void *io_thread_fn(void *arg) {
    AlloyEventLoop *loop = (AlloyEventLoop *)arg;

    while (loop->running) {
        AlloyTask task;

        /* Wait for a task */
        pthread_mutex_lock(&loop->task_mu);
        while (loop->task_count == 0 && loop->running) {
            pthread_cond_wait(&loop->task_cv, &loop->task_mu);
        }
        if (!loop->running) {
            pthread_mutex_unlock(&loop->task_mu);
            break;
        }
        task_dequeue(loop, &task);
        pthread_mutex_unlock(&loop->task_mu);

        if (task.type == ALLOY_TASK_SHUTDOWN) {
            free(task.payload);
            break;
        }

        if (task.type == ALLOY_TASK_TIMER) {
            /* Sleep for delay_ms then post completion */
            struct timespec ts = {
                .tv_sec  = task.delay_ms / 1000,
                .tv_nsec = (task.delay_ms % 1000) * 1000000L,
            };
            nanosleep(&ts, NULL);
            post_completion(loop, task.task_id, ALLOY_COMPLETION_TIMER,
                            strdup("null"));
            free(task.payload);
            continue;
        }

        /* ALLOY_TASK_IPC_SEND: send to sidecar, wait for response */
        AlloyIPCChannel *ch = (AlloyIPCChannel *)loop->ipc_channel;
        if (!ch) {
            post_completion(loop, task.task_id, ALLOY_COMPLETION_ERROR,
                            strdup("{\"error\":\"IPC channel not connected\"}"));
            free(task.payload);
            continue;
        }

        int send_rc = alloy_ipc_send(ch, ALLOY_MSG_WEB_API_REQUEST,
                                      (uint8_t *)task.payload, task.payload_len,
                                      "" /* cap_id embedded in payload */);
        free(task.payload);

        if (send_rc != 0) {
            post_completion(loop, task.task_id, ALLOY_COMPLETION_ERROR,
                            strdup("{\"error\":\"IPC send failed\"}"));
            continue;
        }

        /* Blocking recv — this is fine because we're on the I/O thread */
        AlloyMsg resp;
        if (alloy_ipc_recv(ch, &resp) != 0) {
            post_completion(loop, task.task_id, ALLOY_COMPLETION_ERROR,
                            strdup("{\"error\":\"IPC recv failed\"}"));
            continue;
        }

        /* result is the decrypted payload — transfer ownership */
        char *result = (char *)resp.payload; /* heap-allocated by ipc_recv */
        post_completion(loop, task.task_id, ALLOY_COMPLETION_IPC_RESPONSE, result);
    }

    return NULL;
}

/* ── Public API ─────────────────────────────────────────────────────────── */

AlloyEventLoop *alloy_loop_create(void) {
    AlloyEventLoop *loop = calloc(1, sizeof(AlloyEventLoop));
    if (!loop) return NULL;

    pthread_mutex_init(&loop->task_mu, NULL);
    pthread_cond_init(&loop->task_cv, NULL);
    pthread_mutex_init(&loop->comp_mu, NULL);
    pthread_cond_init(&loop->comp_cv, NULL);

    loop->running      = true;
    loop->next_task_id = 1;

    if (pthread_create(&loop->io_thread, NULL, io_thread_fn, loop) != 0) {
        pthread_mutex_destroy(&loop->task_mu);
        pthread_cond_destroy(&loop->task_cv);
        pthread_mutex_destroy(&loop->comp_mu);
        pthread_cond_destroy(&loop->comp_cv);
        free(loop);
        return NULL;
    }

    return loop;
}

void alloy_loop_destroy(AlloyEventLoop *loop) {
    if (!loop) return;

    /* Signal shutdown */
    loop->running = false;
    AlloyTask shutdown = { .type = ALLOY_TASK_SHUTDOWN };
    pthread_mutex_lock(&loop->task_mu);
    task_enqueue(loop, &shutdown);
    pthread_cond_signal(&loop->task_cv);
    pthread_mutex_unlock(&loop->task_mu);

    pthread_join(loop->io_thread, NULL);

    /* Drain and free any remaining completions */
    AlloyCompletion c;
    pthread_mutex_lock(&loop->comp_mu);
    while (comp_dequeue(loop, &c)) free(c.result);
    pthread_mutex_unlock(&loop->comp_mu);

    pthread_mutex_destroy(&loop->task_mu);
    pthread_cond_destroy(&loop->task_cv);
    pthread_mutex_destroy(&loop->comp_mu);
    pthread_cond_destroy(&loop->comp_cv);
    free(loop);
}

uint32_t alloy_loop_dispatch(AlloyEventLoop *loop,
                              const char *payload, uint32_t payload_len) {
    uint32_t id = loop->next_task_id++;

    char *buf = malloc(payload_len + 1);
    if (!buf) return 0;
    memcpy(buf, payload, payload_len);
    buf[payload_len] = '\0';

    AlloyTask task = {
        .type        = ALLOY_TASK_IPC_SEND,
        .task_id     = id,
        .payload     = buf,
        .payload_len = payload_len,
    };

    pthread_mutex_lock(&loop->task_mu);
    if (!task_enqueue(loop, &task)) {
        pthread_mutex_unlock(&loop->task_mu);
        free(buf);
        return 0;
    }
    pthread_cond_signal(&loop->task_cv);
    pthread_mutex_unlock(&loop->task_mu);

    return id;
}

uint32_t alloy_loop_set_timer(AlloyEventLoop *loop, uint32_t delay_ms) {
    uint32_t id = loop->next_task_id++;

    AlloyTask task = {
        .type     = ALLOY_TASK_TIMER,
        .task_id  = id,
        .delay_ms = delay_ms,
    };

    pthread_mutex_lock(&loop->task_mu);
    if (!task_enqueue(loop, &task)) {
        pthread_mutex_unlock(&loop->task_mu);
        return 0;
    }
    pthread_cond_signal(&loop->task_cv);
    pthread_mutex_unlock(&loop->task_mu);

    return id;
}

int alloy_loop_drain(AlloyEventLoop *loop,
                     AlloyCompletion *out, int max_out) {
    int n = 0;
    pthread_mutex_lock(&loop->comp_mu);
    while (n < max_out && comp_dequeue(loop, &out[n])) n++;
    pthread_mutex_unlock(&loop->comp_mu);
    return n;
}

int alloy_loop_wait_and_drain(AlloyEventLoop *loop,
                               AlloyCompletion *out, int max_out) {
    pthread_mutex_lock(&loop->comp_mu);
    while (loop->comp_count == 0 && loop->running) {
        pthread_cond_wait(&loop->comp_cv, &loop->comp_mu);
    }
    int n = 0;
    while (n < max_out && comp_dequeue(loop, &out[n])) n++;
    pthread_mutex_unlock(&loop->comp_mu);
    return n;
}

/**
 * AlloyScript — Event Loop
 *
 * Single-threaded JS execution model with a background I/O thread:
 *
 *   JS thread (main)          I/O thread (background)
 *   ─────────────────         ──────────────────────────
 *   gen.next()                waits on IPC socket (blocking read)
 *   → yields task_id          ← receives response
 *   → parks coroutine         → posts AlloyCompletion to queue
 *   → drains other tasks      → signals condvar
 *   ← wakes on condvar
 *   → resumes coroutine with result
 *
 * The JS thread never blocks on I/O. It only blocks on the condvar when
 * the completion queue is empty AND there are pending coroutines — i.e.
 * it has nothing else to do.
 *
 * Concurrency model:
 *   - Multiple coroutines can be in-flight simultaneously.
 *   - Each coroutine is identified by a uint32_t task_id.
 *   - The I/O thread tags every IPC response with the task_id.
 *   - The JS thread resumes the matching coroutine when its completion arrives.
 *   - Promise.all works correctly: all fetches are dispatched before any
 *     response is awaited.
 */
#ifndef ALLOY_EVENT_LOOP_H
#define ALLOY_EVENT_LOOP_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>
#include <pthread.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ── Completion (I/O thread → JS thread) ───────────────────────────────── */

typedef enum {
    ALLOY_COMPLETION_IPC_RESPONSE = 1,  /* Web API call finished          */
    ALLOY_COMPLETION_TIMER        = 2,  /* setTimeout/setInterval fired   */
    ALLOY_COMPLETION_ERROR        = 3,  /* I/O thread encountered error   */
} AlloyCompletionType;

typedef struct {
    AlloyCompletionType type;
    uint32_t            task_id;    /* matches the pending coroutine      */
    char               *result;     /* heap-allocated JSON; caller frees  */
} AlloyCompletion;

/* ── Pending task (JS thread → I/O thread) ─────────────────────────────── */

typedef enum {
    ALLOY_TASK_IPC_SEND   = 1,  /* send an IPC request to the sidecar  */
    ALLOY_TASK_TIMER      = 2,  /* schedule a timer                    */
    ALLOY_TASK_SHUTDOWN   = 255,
} AlloyTaskType;

typedef struct {
    AlloyTaskType type;
    uint32_t      task_id;
    char         *payload;      /* heap-allocated; I/O thread frees    */
    uint32_t      payload_len;
    uint32_t      delay_ms;     /* for TIMER tasks                     */
} AlloyTask;

/* ── Event loop ─────────────────────────────────────────────────────────── */

#define ALLOY_LOOP_QUEUE_CAP 256

typedef struct AlloyEventLoop AlloyEventLoop;

struct AlloyEventLoop {
    /* Task queue: JS thread enqueues, I/O thread dequeues */
    AlloyTask        task_queue[ALLOY_LOOP_QUEUE_CAP];
    uint32_t         task_head;
    uint32_t         task_tail;
    uint32_t         task_count;
    pthread_mutex_t  task_mu;
    pthread_cond_t   task_cv;   /* signals I/O thread when task arrives */

    /* Completion queue: I/O thread enqueues, JS thread dequeues */
    AlloyCompletion  comp_queue[ALLOY_LOOP_QUEUE_CAP];
    uint32_t         comp_head;
    uint32_t         comp_tail;
    uint32_t         comp_count;
    pthread_mutex_t  comp_mu;
    pthread_cond_t   comp_cv;   /* signals JS thread when completion arrives */

    pthread_t        io_thread;
    bool             running;
    uint32_t         next_task_id;  /* monotonic, JS thread only */

    /* Opaque pointer to the IPC channel — set by the web plugin */
    void            *ipc_channel;
};

/* ── API ────────────────────────────────────────────────────────────────── */

/** Create and start the event loop (spawns I/O thread). */
AlloyEventLoop *alloy_loop_create(void);

/** Stop the I/O thread and free all resources. */
void alloy_loop_destroy(AlloyEventLoop *loop);

/**
 * Enqueue an IPC send task from the JS thread.
 * Returns a task_id that will appear in the matching AlloyCompletion.
 * Non-blocking — returns immediately.
 */
uint32_t alloy_loop_dispatch(AlloyEventLoop *loop,
                              const char *payload, uint32_t payload_len);

/**
 * Enqueue a timer task.
 * Returns a task_id. The completion fires after delay_ms milliseconds.
 */
uint32_t alloy_loop_set_timer(AlloyEventLoop *loop, uint32_t delay_ms);

/**
 * Drain all available completions into the out array.
 * Returns the number of completions written (≤ max_out).
 * Non-blocking — returns 0 immediately if queue is empty.
 * Caller must free each completion's result string.
 */
int alloy_loop_drain(AlloyEventLoop *loop,
                     AlloyCompletion *out, int max_out);

/**
 * Block until at least one completion is available, then drain.
 * Used when the JS thread has no runnable coroutines.
 */
int alloy_loop_wait_and_drain(AlloyEventLoop *loop,
                               AlloyCompletion *out, int max_out);

#ifdef __cplusplus
}
#endif

#endif /* ALLOY_EVENT_LOOP_H */

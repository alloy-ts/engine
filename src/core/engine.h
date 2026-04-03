#ifndef ALLOYSCRIPT_ENGINE_H
#define ALLOYSCRIPT_ENGINE_H

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>
#include "mquickjs.h"
#include "webview/webview.h"
#include "js_engine.h"

#ifdef __cplusplus
extern "C" {
#endif

#define ALLOY_EVENT_QUEUE_CAPACITY 4096

typedef struct {
    uint32_t    type;
    char       *name;
    JSValue     listener;
    JSValue     args[8];
    uint8_t     argc;
} AlloyEvent;

typedef struct {
    AlloyEvent  events[ALLOY_EVENT_QUEUE_CAPACITY];
    uint32_t    head;
    uint32_t    tail;
    uint32_t    count;
} AlloyEventQueue;

typedef struct {
    const char *event_name;
    JSValue     listeners[8];
    uint8_t     count;
} AlloyListenerEntry;

typedef struct {
    uint64_t next_outbound_seq;
    uint64_t last_inbound_seq;
    unsigned char session_key[32];
} AlloyIPCState;

typedef struct alloy_runtime {
    AlloyJSEngine    *mqjs_engine;
    AlloyJSEngine    *webview_engine;

    JSContext        *mqjs_ctx; // Legacy, move inside engine soon
    webview_t         webview;

    AlloyIPCState     ipc_state;
    void             *mem_buf;
    size_t            js_heap_size;
    int               js_heap_prot;

    AlloyEventQueue   event_queue;
    AlloyListenerEntry listener_registry[32];

    JSValue           next_tick_queue[256];
    int               next_tick_count;

    bool              shutdown;
} alloy_runtime_t;

alloy_runtime_t *alloy_engine_create(void);
void alloy_run(alloy_runtime_t *rt);
void alloy_engine_destroy(alloy_runtime_t *rt);

void alloy_bind_global(alloy_runtime_t *rt, const char *name, AlloyBindHandler handler, void *opaque);
void alloy_event_emit(alloy_runtime_t *rt, const char *name, JSValue arg);
AlloyEvent *alloy_event_dequeue(alloy_runtime_t *rt);
void alloy_poll_io(alloy_runtime_t *rt);
void alloy_drain_next_tick(alloy_runtime_t *rt);

#ifdef __cplusplus
}
#endif

#endif

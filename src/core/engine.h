#ifndef ALLOYSCRIPT_ENGINE_H
#define ALLOYSCRIPT_ENGINE_H

#include <stddef.h>
#include <stdint.h>
#include "mquickjs.h"
#ifdef MOCK_WEBVIEW
#include "webview_mock.h"
#else
#include "webview/webview.h"
#endif
#include "js_engine.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct alloy_ipc_state {
    unsigned char session_key[32];
    uint64_t next_outbound_seq;
    uint64_t last_inbound_seq;
} alloy_ipc_state_t;

typedef struct {
    char *event_name;
    JSValue listeners[8];
    int count;
} AlloyListenerEntry;

typedef struct {
    char *name;
    JSValue listener;
    JSValue args[1];
    int argc;
} AlloyEvent;

#define ALLOY_EVENT_QUEUE_CAPACITY 128
typedef struct {
    AlloyEvent events[ALLOY_EVENT_QUEUE_CAPACITY];
    int head;
    int tail;
    int count;
} AlloyEventQueue;

typedef struct alloy_runtime {
    JSContext *mqjs_ctx;
    webview_t webview;

    AlloyJSEngine *core_engine;    // MicroQuickJS
    AlloyJSEngine *sidecar_engine; // WebView

    alloy_ipc_state_t ipc_state;
    void *mem_buf;
    size_t js_heap_size;
    int shutdown;

    AlloyEventQueue event_queue;
    AlloyListenerEntry listener_registry[32];
    JSValue next_tick_queue[256];
    int next_tick_count;
} alloy_runtime_t;

typedef alloy_engine_bind_cb alloy_bind_fn_t;

alloy_runtime_t *alloy_engine_create(void);
void alloy_engine_destroy(alloy_runtime_t *rt);
void alloy_run(alloy_runtime_t *rt);

void alloy_return(alloy_runtime_t *rt, const char *id, int status, const char *result);
JSValue alloy_secure_eval(alloy_runtime_t *rt, const char *script);

/* Legacy support - should be migrated to new abstraction */
void alloy_bind_global(alloy_runtime_t *rt, const char *name, alloy_bind_fn_t fn, void *arg);

#ifdef __cplusplus
}
#endif

#endif // ALLOYSCRIPT_ENGINE_H

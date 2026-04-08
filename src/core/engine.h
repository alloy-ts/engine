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

typedef enum {
    ALLOY_TARGET_NATIVE,
    ALLOY_TARGET_BROWSER
} AlloyTarget;

typedef struct {
    uint32_t type;
    char *name;
    JSValue listener;
    JSValue args[8];
    uint8_t argc;
} AlloyEvent;

typedef struct {
    AlloyEvent events[4096];
    uint32_t head;
    uint32_t tail;
    uint32_t count;
} AlloyEventQueue;

typedef struct {
    const char *event_name;
    JSValue listeners[8];
    uint8_t count;
} AlloyListenerEntry;

typedef struct {
    uint8_t session_key[32];
    uint64_t next_outbound_seq;
    uint64_t last_inbound_seq;
} AlloyIPCState;

typedef struct alloy_runtime {
    AlloyJSEngine *mqjs_engine;
    AlloyJSEngine *webview_engine;

    JSContext *mqjs_ctx;
    webview_t webview;

    void *js_heap;
    size_t js_heap_size;
    int js_heap_prot;

    AlloyIPCState ipc_state;
    char *latest_ipc_response;

    AlloyEventQueue event_queue;
    AlloyListenerEntry listener_registry[32];

    JSValue next_tick_queue[256];
    uint32_t next_tick_count;

    bool shutdown;
    AlloyTarget target;
} alloy_runtime_t;

alloy_runtime_t *alloy_engine_create(void);
void alloy_run(alloy_runtime_t *rt);
void alloy_engine_destroy(alloy_runtime_t *rt);

void alloy_bind_global(alloy_runtime_t *rt, const char *name, AlloyBindHandler handler, void *opaque);
JSValue alloy_secure_eval(alloy_runtime_t *rt, const char *script);

#ifdef __cplusplus
}
#endif

#endif // ALLOYSCRIPT_ENGINE_H

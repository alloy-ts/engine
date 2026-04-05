#ifndef ALLOYSCRIPT_ENGINE_H
#define ALLOYSCRIPT_ENGINE_H

#include <stddef.h>
#include <stdbool.h>
#include <stdint.h>
#include "mquickjs.h"
#include "webview/webview.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct AlloyJSEngine AlloyJSEngine;

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

typedef struct alloy_runtime {
    JSContext *mqjs_ctx;
    webview_t webview;
    unsigned char session_key[32];
    void *mem_buf;

    AlloyJSEngine *mqjs_engine;
    AlloyJSEngine *webview_engine;

    struct {
        uint64_t next_outbound_seq;
        uint64_t last_inbound_seq;
        unsigned char session_key[32];
    } ipc_state;

    char *latest_ipc_response;
    bool shutdown;

    size_t js_heap_size;
    void *js_heap;
    int js_heap_prot;

    AlloyListenerEntry listener_registry[32];
    AlloyEventQueue event_queue;

    JSValue next_tick_queue[256];
    uint32_t next_tick_count;
} alloy_runtime_t;

void alloy_mqjs_bind_func(JSContext *ctx, const char *name, JSCFunction *func, void *opaque);
void alloy_mqjs_bind_global(JSContext *ctx, const char *name, JSValue val);

typedef void (*alloy_bind_fn_t)(const char *id, const char *req, void *arg);

alloy_runtime_t *alloy_init(size_t mqjs_mem_size);
void alloy_run(alloy_runtime_t *rt);
void alloy_destroy(alloy_runtime_t *rt);
alloy_runtime_t *alloy_engine_create(void);
void alloy_engine_destroy(alloy_runtime_t *rt);

/**
 * Binds a global function to both MicroQuickJS and WebView.
 * The callback will receive JSON-encoded arguments in 'req'.
 */
void alloy_bind_global(alloy_runtime_t *rt, const char *name, alloy_bind_fn_t fn, void *arg);

/**
 * Returns a result from a binding.
 */
void alloy_return(alloy_runtime_t *rt, const char *id, int status, const char *result);


/**
 * Securely evaluates JavaScript in the MicroQuickJS engine.
 */
JSValue alloy_secure_eval(alloy_runtime_t *rt, const char *script);

/* WebView Browser API bindings for MQJS */
JSValue alloy_browser_proxy_fetch(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue alloy_browser_proxy_crypto(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue alloy_browser_proxy_url(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue alloy_browser_proxy_text_encoder(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue alloy_browser_proxy_text_decoder(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue alloy_browser_proxy_performance(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);

JSValue alloy_runtime_spawn(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue alloy_runtime_spawn_sync(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue alloy_runtime_secure_eval_internal(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);

#ifdef __cplusplus
}
#endif

#endif // ALLOYSCRIPT_H

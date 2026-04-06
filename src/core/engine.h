#ifndef ALLOYSCRIPT_ENGINE_H
#define ALLOYSCRIPT_ENGINE_H

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>
#include "mquickjs.h"
#include "webview/webview.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef enum {
    ALLOY_TARGET_NATIVE = 0,
    ALLOY_TARGET_BROWSER = 1,
} AlloyTarget;

typedef struct {
    uint8_t session_key[32];
    uint64_t next_outbound_seq;
    uint64_t last_inbound_seq;
} AlloyIPCState;

typedef struct {
    uint32_t type;
    char *name;
    JSValue listener;
    JSValue args[8];
    uint8_t argc;
} AlloyEvent;

typedef struct {
    AlloyEvent events[4096];
    int head;
    int tail;
    int count;
} AlloyEventQueue;

typedef struct {
    char *event_name;
    JSValue listeners[8];
    uint8_t count;
} AlloyListenerEntry;

typedef struct alloy_runtime {
    JSContext *mqjs_ctx;
    webview_t webview;
    AlloyTarget target;
    AlloyIPCState ipc_state;

    void *js_heap;
    size_t js_heap_size;
    int js_heap_prot;

    char *latest_ipc_response;
    AlloyEventQueue event_queue;
    AlloyListenerEntry listener_registry[32];

    bool shutdown;
    void *mem_buf;
} alloy_runtime_t;

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

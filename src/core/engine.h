#ifndef ALLOYSCRIPT_ENGINE_H
#define ALLOYSCRIPT_ENGINE_H

#include <stddef.h>
#include "mquickjs.h"
#include "webview/webview.h"

#ifdef __cplusplus
extern "C" {
#endif

/* Requirement 3.1: Event structs */
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

    /* Requirement 3.1: Event-related fields */
    AlloyEventQueue event_queue;
    AlloyListenerEntry listener_registry[32];

    /* Integration with configuration */
    /* Requirement 1.11: Add browser target mode */
    int target;               /* 0: local-first, 1: server-first */
    bool wasm_edge_enabled;
    bool use_containers;
    char **shell_whitelist;
    int shell_whitelist_len;
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

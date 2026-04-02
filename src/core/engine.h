#ifndef ALLOYSCRIPT_ENGINE_H
#define ALLOYSCRIPT_ENGINE_H

#include <stddef.h>
#include "mquickjs.h"
#include "webview/webview.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct alloy_runtime {
    JSContext *mqjs_ctx;
    webview_t webview;
    unsigned char session_key[32];
    void *mem_buf;
} alloy_runtime_t;

typedef void (*alloy_bind_fn_t)(const char *id, const char *req, void *arg);

alloy_runtime_t *alloy_init(size_t mqjs_mem_size);
void alloy_run(alloy_runtime_t *rt);
void alloy_destroy(alloy_runtime_t *rt);

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

#ifdef __cplusplus
}
#endif

#endif // ALLOYSCRIPT_H

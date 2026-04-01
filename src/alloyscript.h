
#ifndef ALLOYSCRIPT_H
#define ALLOYSCRIPT_H

#include "../packages/mquickjs/mquickjs.h"
#include "../packages/webview/core/include/webview/webview.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct alloy_runtime alloy_runtime_t;

// Context for a binding call
typedef struct {
    alloy_runtime_t *rt;
    const char *id; // for async return if needed
    int is_mqjs;    // true if called from MQJS, false if from WebView
} alloy_context_t;

// Unified binding function type
typedef JSValue (*alloy_binding_fn_t)(JSContext *ctx, JSValue this_val, int argc, JSValue *argv, void *user_data);

alloy_runtime_t* alloy_init(int debug);
void alloy_destroy(alloy_runtime_t *rt);

/**
 * Binds a global function to both the WebView and MicroQuickJS environments.
 * name: The name of the function (e.g. "secureEval")
 * fn: The C function to call
 * arg_count: Number of arguments
 * user_data: Opaque pointer passed to the function
 */
int alloy_bind_global(alloy_runtime_t *rt, const char *name, alloy_binding_fn_t fn, int arg_count, void *user_data);

/**
 * Runs the main event loop.
 */
void alloy_run(alloy_runtime_t *rt);

/**
 * Securely evaluates JS code in the MicroQuickJS engine.
 */
JSValue alloy_secure_eval(alloy_runtime_t *rt, const char *script, const char *filename);

JSContext* alloy_get_context(alloy_runtime_t *rt);

#ifdef __cplusplus
}
#endif

#endif // ALLOYSCRIPT_H

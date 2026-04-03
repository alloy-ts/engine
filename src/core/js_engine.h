#ifndef ALLOYSCRIPT_JS_ENGINE_H
#define ALLOYSCRIPT_JS_ENGINE_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Callback type for JS -> C calls.
 * @param id Unique request ID (for async returns if needed)
 * @param req JSON-encoded arguments
 * @param arg Opaque user data
 * @return JSON-encoded result (for synchronous returns, caller must free)
 */
typedef char* (*alloy_engine_bind_cb)(const char *id, const char *req, void *arg);

/**
 * Unified JS Engine abstraction vtable.
 */
typedef struct AlloyJSEngine {
    void *instance; // Engine-specific instance (e.g., JSContext* or webview_t)
    void *priv;     // Engine-specific private data

    /**
     * Calls a JS function by name with JSON arguments.
     * Returns a JSON-encoded string (caller must free).
     */
    char* (*call_fn)(struct AlloyJSEngine *engine, const char *name, const char *args_json);

    /**
     * Evaluates a JS source string.
     * Returns 0 on success, non-zero on error.
     */
    int (*eval_fn)(struct AlloyJSEngine *engine, const char *script);

    /**
     * Binds a C handler to a JS global name.
     */
    void (*bind_fn)(struct AlloyJSEngine *engine, const char *name, alloy_engine_bind_cb cb, void *arg);

    /**
     * Emits a named event into the JS runtime with JSON data.
     */
    void (*emit_fn)(struct AlloyJSEngine *engine, const char *name, const char *event_json);

    /**
     * Tears down the engine instance.
     */
    void (*destroy_fn)(struct AlloyJSEngine *engine);
} AlloyJSEngine;

#ifdef __cplusplus
}
#endif

#endif // ALLOYSCRIPT_JS_ENGINE_H

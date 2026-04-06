/**
 * AlloyScript — AlloyJSEngine vtable
 *
 * Abstracts MicroQuickJS and WebView behind five uniform function pointers
 * with a strict JSON string boundary. No engine-specific type (JSValue,
 * webview_t, JSContext*) ever crosses this interface.
 *
 * Adding a new engine target requires implementing exactly five functions
 * in a new adapter file — nothing else.
 */
#ifndef ALLOY_JS_ENGINE_H
#define ALLOY_JS_ENGINE_H

#include <stddef.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ── Error codes ────────────────────────────────────────────────────────── */
#define ALLOY_OK              0
#define ALLOY_ERR_NULL_ARG   -1
#define ALLOY_ERR_EVAL       -2
#define ALLOY_ERR_CALL       -3
#define ALLOY_ERR_BIND       -4
#define ALLOY_ERR_EMIT       -5
#define ALLOY_ERR_OOM        -6
#define ALLOY_ERR_DESTROYED  -7

/* ── Handler callback (JS→C binding) ───────────────────────────────────── */
/**
 * Called when JS invokes a bound function.
 * @param name       The bound function name.
 * @param args_json  JSON-encoded argument array (e.g. "[1, \"hello\"]").
 * @param userdata   Opaque pointer passed at bind time.
 * @return           Heap-allocated JSON result string (caller frees), or NULL on OOM.
 *                   Return "{}" for void results.
 */
typedef char *(*alloy_handler_fn)(const char *name, const char *args_json, void *userdata);

/* ── Engine vtable ──────────────────────────────────────────────────────── */
typedef struct AlloyJSEngine {
    void *engine;  /* opaque: JSContext* for MQJS, webview_t for WebView */

    /**
     * call_fn — Call a named JS function with JSON-encoded arguments.
     * @param fn_name    Global function name (e.g. "Alloy.plugins.myPlugin.init").
     * @param args_json  JSON array string (e.g. "[1, true, \"x\"]").
     * @return           Heap-allocated JSON result string (caller frees), or NULL on OOM.
     *                   On exception: returns JSON error object {"error":"<msg>"}.
     */
    char *(*call_fn)(struct AlloyJSEngine *eng, const char *fn_name, const char *args_json);

    /**
     * eval_fn — Evaluate a JS source string.
     * @param source     UTF-8 JS source code.
     * @param filename   Logical filename for error messages (e.g. "<bootstrap>").
     * @return           Heap-allocated JSON result string (caller frees), or NULL on OOM.
     */
    char *(*eval_fn)(struct AlloyJSEngine *eng, const char *source, const char *filename);

    /**
     * bind_fn — Register a C handler as a named global JS function.
     * The handler is called synchronously when JS invokes the function.
     * @param name       Global name to bind (e.g. "__alloy_fetch").
     * @param handler    C callback.
     * @param userdata   Passed through to handler on each call.
     * @return           ALLOY_OK or ALLOY_ERR_*.
     */
    int (*bind_fn)(struct AlloyJSEngine *eng, const char *name,
                   alloy_handler_fn handler, void *userdata);

    /**
     * emit_fn — Dispatch a named event to JS (calls __alloy_emit(name, payload)).
     * @param event_name  Event name string.
     * @param payload_json JSON-encoded event payload.
     * @return            ALLOY_OK or ALLOY_ERR_*.
     */
    int (*emit_fn)(struct AlloyJSEngine *eng, const char *event_name,
                   const char *payload_json);

    /**
     * destroy_fn — Tear down the engine and free all resources.
     * After this call, the AlloyJSEngine pointer is invalid.
     */
    void (*destroy_fn)(struct AlloyJSEngine *eng);

    bool destroyed;  /* set to true after destroy_fn; guards vtable wrappers */
} AlloyJSEngine;

/* ── Inline vtable wrappers (NULL-guard + destroyed-guard) ─────────────── */

static inline char *alloy_engine_call(AlloyJSEngine *eng,
                                       const char *fn_name,
                                       const char *args_json) {
    if (!eng || eng->destroyed || !eng->call_fn) return NULL;
    return eng->call_fn(eng, fn_name, args_json ? args_json : "[]");
}

static inline char *alloy_engine_eval(AlloyJSEngine *eng,
                                       const char *source,
                                       const char *filename) {
    if (!eng || eng->destroyed || !eng->eval_fn) return NULL;
    return eng->eval_fn(eng, source, filename ? filename : "<eval>");
}

static inline int alloy_engine_bind(AlloyJSEngine *eng, const char *name,
                                     alloy_handler_fn handler, void *userdata) {
    if (!eng || eng->destroyed || !eng->bind_fn) return ALLOY_ERR_DESTROYED;
    return eng->bind_fn(eng, name, handler, userdata);
}

static inline int alloy_engine_emit(AlloyJSEngine *eng,
                                     const char *event_name,
                                     const char *payload_json) {
    if (!eng || eng->destroyed || !eng->emit_fn) return ALLOY_ERR_DESTROYED;
    return eng->emit_fn(eng, event_name, payload_json ? payload_json : "null");
}

static inline void alloy_engine_destroy(AlloyJSEngine *eng) {
    if (!eng || eng->destroyed) return;
    eng->destroyed = true;
    if (eng->destroy_fn) eng->destroy_fn(eng);
}

#ifdef __cplusplus
}
#endif

#endif /* ALLOY_JS_ENGINE_H */

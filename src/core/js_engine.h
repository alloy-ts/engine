#ifndef ALLOY_JS_ENGINE_H
#define ALLOY_JS_ENGINE_H

#include <stddef.h>

/* Error codes returned by vtable wrappers and bind_fn/emit_fn */
#define ALLOY_OK                  0
#define ALLOY_ERR_NULL_FN        -1   /* function pointer is NULL */
#define ALLOY_ERR_INVALID_JSON   -2   /* args_json is not valid JSON */
#define ALLOY_ERR_NOT_FOUND      -3   /* named function/event not found */
#define ALLOY_ERR_ENGINE_DEAD    -4   /* engine has been destroyed */
#define ALLOY_ERR_NAME_CONFLICT  -5   /* global name already registered */
#define ALLOY_ERR_INIT_FAILED    -6   /* on_init hook returned failure */
#define ALLOY_ERR_CYCLE          -7   /* dependency cycle detected */
#define ALLOY_ERR_MISSING_DEP    -8   /* declared dependency not found */
#define ALLOY_ERR_MANIFEST       -9   /* alloy-plugin.toml parse error */
#define ALLOY_ERR_DLOPEN        -10   /* dlopen/dlsym failure */

/*
 * alloy_handler_fn — C callback invoked when JS calls a bound global.
 */
typedef char *(*alloy_handler_fn)(const char *args_json, void *userdata);

/*
 * AlloyJSEngine — the five-pointer vtable abstracting any JS runtime.
 */
typedef struct AlloyJSEngine {
    void *engine;   /* opaque adapter-private context (JSContext*, webview_t, …) */

    /* C→JS call: invoke named JS function with JSON-array args.
     * Returns heap-allocated JSON string (caller frees), or NULL on OOM. */
    char *(*call_fn)(void *engine, const char *fn_name, const char *args_json);

    /* Evaluate JS source string.
     * Returns heap-allocated JSON string of last value (caller frees), or NULL. */
    char *(*eval_fn)(void *engine, const char *source);

    /* Register C handler as named JS global.
     * Returns ALLOY_OK or negative error code. */
    int   (*bind_fn)(void *engine, const char *name,
                     alloy_handler_fn handler, void *userdata);

    /* Fire named event into JS runtime with JSON payload.
     * Returns ALLOY_OK or negative error code. */
    int   (*emit_fn)(void *engine, const char *event_name,
                     const char *payload_json);

    /* Tear down engine and release all resources. */
    void  (*destroy_fn)(void *engine);
} AlloyJSEngine;

/* ── Public inline wrappers ─────────────────────────────────────────────── */

static inline char *alloy_engine_call(AlloyJSEngine *e,
                                       const char *fn_name,
                                       const char *args_json) {
    if (!e || !e->call_fn) return NULL;
    return e->call_fn(e->engine, fn_name, args_json);
}

static inline char *alloy_engine_eval(AlloyJSEngine *e, const char *source) {
    if (!e || !e->eval_fn) return NULL;
    return e->eval_fn(e->engine, source);
}

static inline int alloy_engine_bind(AlloyJSEngine *e, const char *name,
                                     alloy_handler_fn handler, void *userdata) {
    if (!e || !e->bind_fn) return ALLOY_ERR_NULL_FN;
    return e->bind_fn(e->engine, name, handler, userdata);
}

static inline int alloy_engine_emit(AlloyJSEngine *e, const char *event_name,
                                     const char *payload_json) {
    if (!e || !e->emit_fn) return ALLOY_ERR_NULL_FN;
    return e->emit_fn(e->engine, event_name, payload_json);
}

static inline void alloy_engine_destroy(AlloyJSEngine *e) {
    if (!e || !e->destroy_fn) return;
    e->destroy_fn(e->engine);
}

#endif /* ALLOY_JS_ENGINE_H */

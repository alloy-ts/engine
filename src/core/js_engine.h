#ifndef ALLOY_JS_ENGINE_H
#define ALLOY_JS_ENGINE_H

#include <stddef.h>

typedef struct AlloyJSEngine AlloyJSEngine;

/*
 * Handler called when JS invokes a bound C function.
 * opaque:       the pointer registered with bind_fn
 * args_json:    JSON-encoded argument array, e.g. ["url", {"method":"GET"}]
 * out_result:   caller-allocated buffer; handler writes JSON result here
 * result_size:  size of out_result buffer
 */
typedef void (*AlloyBindHandler)(void *opaque,
                                 const char *args_json,
                                 char *out_result,
                                 size_t result_size);

struct AlloyJSEngine {
    /*
     * Call a named JS function with JSON-encoded args.
     * out_result is heap-allocated; caller must free().
     */
    int (*call_fn)(AlloyJSEngine *engine,
                   const char *name,
                   const char *args_json,
                   char **out_result);

    /*
     * Evaluate a JS source string.
     * out_result is heap-allocated; caller must free().
     */
    int (*eval_fn)(AlloyJSEngine *engine,
                   const char *source,
                   const char *filename,
                   char **out_result);

    /*
     * Register a C handler under a JS global name.
     */
    int (*bind_fn)(AlloyJSEngine *engine,
                   const char *name,
                   AlloyBindHandler handler,
                   void *opaque);

    /*
     * Fire a named event with a JSON payload into the JS runtime.
     */
    int (*emit_fn)(AlloyJSEngine *engine,
                   const char *event_name,
                   const char *payload_json);

    /*
     * Tear down the engine and free all resources.
     */
    void (*destroy_fn)(AlloyJSEngine *engine);

    /* Engine identity — for diagnostics only */
    const char *name;  /* "mqjs" or "webview" */
};

/* Convenience macros */
#define alloy_engine_call(e, name, args, out)   (e)->call_fn(e, name, args, out)
#define alloy_engine_eval(e, src, file, out)    (e)->eval_fn(e, src, file, out)
#define alloy_engine_bind(e, name, fn, opaque)  (e)->bind_fn(e, name, fn, opaque)
#define alloy_engine_emit(e, ev, payload)       (e)->emit_fn(e, ev, payload)
#define alloy_engine_destroy(e)                 (e)->destroy_fn(e)

#endif

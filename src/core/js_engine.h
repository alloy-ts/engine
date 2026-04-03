#ifndef ALLOY_JS_ENGINE_H
#define ALLOY_JS_ENGINE_H

#include <stddef.h>

typedef struct AlloyJSEngine AlloyJSEngine;

/*
 * Handler called when JS invokes a bound C function.
 */
typedef void (*AlloyBindHandler)(void *opaque,
                                 const char *args_json,
                                 char *out_result,
                                 size_t result_size);

struct AlloyJSEngine {
    int (*call_fn)(AlloyJSEngine *engine,
                   const char *name,
                   const char *args_json,
                   char **out_result);

    int (*eval_fn)(AlloyJSEngine *engine,
                   const char *source,
                   const char *filename,
                   char **out_result);

    int (*bind_fn)(AlloyJSEngine *engine,
                   const char *name,
                   AlloyBindHandler handler,
                   void *opaque);

    int (*emit_fn)(AlloyJSEngine *engine,
                   const char *event_name,
                   const char *payload_json);

    void (*destroy_fn)(AlloyJSEngine *engine);

    const char *name;
};

#define alloy_engine_call(e, name, args, out)   (e)->call_fn(e, name, args, out)
#define alloy_engine_eval(e, src, file, out)    (e)->eval_fn(e, src, file, out)
#define alloy_engine_bind(e, name, fn, opaque)  (e)->bind_fn(e, name, fn, opaque)
#define alloy_engine_emit(e, ev, payload)       (e)->emit_fn(e, ev, payload)
#define alloy_engine_destroy(e)                 (e)->destroy_fn(e)

#endif

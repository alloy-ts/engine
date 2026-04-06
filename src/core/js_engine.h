#ifndef ALLOYSCRIPT_JS_ENGINE_H
#define ALLOYSCRIPT_JS_ENGINE_H

#include <stddef.h>

typedef struct AlloyJSEngine AlloyJSEngine;

typedef void (*AlloyBindHandler)(const char *id, const char *req, void *arg);

struct AlloyJSEngine {
    void *engine_ctx;

    /* Requirement 18.2: call_fn - resolve dotted name → JSON result */
    char* (*call_fn)(AlloyJSEngine *e, const char *name, const char *args_json);

    /* Requirement 18.3: bind_fn - register host function */
    void (*bind_fn)(AlloyJSEngine *e, const char *name, AlloyBindHandler handler, void *arg);

    /* Requirement 18.4: eval_fn - evaluate source → JSON result */
    char* (*eval_fn)(AlloyJSEngine *e, const char *source);

    /* Requirement 18.5: emit_fn - dispatch event to JS listeners */
    void (*emit_fn)(AlloyJSEngine *e, const char *name, const char *args_json);

    /* Requirement 18.6: destroy_fn - teardown engine */
    void (*destroy_fn)(AlloyJSEngine *e);
};

/* Factory functions for adapters */
AlloyJSEngine* alloy_mqjs_engine_create(void *js_heap, size_t heap_size);
AlloyJSEngine* alloy_webview_engine_create(void);

#endif

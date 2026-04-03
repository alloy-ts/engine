#ifndef ALLOY_PLUGIN_H
#define ALLOY_PLUGIN_H

#include "js_engine.h"

/*
 * RuntimePlugin — the lifecycle contract for all AlloyScript plugins.
 *
 * All hook pointers are nullable. The host skips NULL hooks without error.
 * on_init returning a negative integer aborts engine initialisation.
 * on_destroy is called in reverse topological order before destroy_fn.
 */
typedef struct RuntimePlugin {
    const char *name;     /* unique plugin identifier, e.g. "alloy:shell" */
    const char *version;  /* semver string, e.g. "1.0.0" */

    /* Called once after engine is fully initialised, before any user script.
     * Use to register JS globals via alloy_engine_bind.
     * Return ALLOY_OK (0) on success, negative to abort engine init. */
    int  (*on_init)(AlloyJSEngine *engine, void *userdata);

    /* Called before each top-level script evaluation.
     * source is the JS source string about to be evaluated. */
    int  (*on_eval_begin)(AlloyJSEngine *engine, const char *source,
                          void *userdata);

    /* Called after each top-level script evaluation completes.
     * result_json is the JSON-serialised result, or {"error":"..."} on failure. */
    int  (*on_eval_end)(AlloyJSEngine *engine, const char *result_json,
                        void *userdata);

    /* Called during engine teardown, before destroy_fn.
     * Release all resources here. */
    void (*on_destroy)(AlloyJSEngine *engine, void *userdata);

    void *userdata;  /* opaque pointer passed to all hooks */
} RuntimePlugin;

/*
 * Symbol that native plugin shared libraries must export.
 * Called by PluginLoadPipeline after dlopen to obtain the RuntimePlugin*.
 */
typedef RuntimePlugin *(*alloy_plugin_init_fn)(void);

#endif /* ALLOY_PLUGIN_H */

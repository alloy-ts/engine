/**
 * AlloyScript — AlloyRuntime public API
 */
#ifndef ALLOY_RUNTIME_H
#define ALLOY_RUNTIME_H

#include "js_engine.h"
#include "capability.h"
#include "config.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct AlloyRuntime AlloyRuntime;

AlloyRuntime  *alloy_engine_create(void);
AlloyRuntime  *alloy_engine_create_with_config(AlloyRuntimeConfig cfg);
void           alloy_engine_destroy_runtime(AlloyRuntime *rt);
AlloyJSEngine *alloy_engine_get(AlloyRuntime *rt);
AlloyCapStore *alloy_engine_get_cap_store(AlloyRuntime *rt);
void           alloy_engine_notify_eval_begin(AlloyRuntime *rt, const char *source);
void           alloy_engine_notify_eval_end(AlloyRuntime *rt, const char *result_json);

/**
 * alloy_runtime_eval — evaluate a script and fire all plugin hooks.
 *
 * This is the only function main_host.c (and any other caller) should use
 * to run user scripts. It fires on_eval_begin before and on_eval_end after,
 * so plugins receive every evaluation automatically.
 *
 * Returns a heap-allocated JSON result string (caller must free), or NULL.
 */
char          *alloy_runtime_eval(AlloyRuntime *rt, const char *source,
                                   const char *filename);

/**
 * Returns the number of loaded plugins.
 */
int alloy_engine_plugin_count(AlloyRuntime *rt);

/**
 * Returns the plugin at index i, or NULL if out of range.
 * The returned pointer is owned by the runtime — do not free it.
 */
const RuntimePlugin *alloy_engine_get_plugin(AlloyRuntime *rt, int i);

/**
 * Looks up a loaded plugin by name (e.g. "alloy:web").
 * Returns NULL if not found.
 */
const RuntimePlugin *alloy_engine_find_plugin(AlloyRuntime *rt, const char *name);

#ifdef __cplusplus
}
#endif

#endif /* ALLOY_RUNTIME_H */

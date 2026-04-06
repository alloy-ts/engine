/**
 * AlloyScript — RuntimePlugin interface
 *
 * Every subsystem (including core stdlib modules) is a plugin.
 * Plugins implement up to four nullable lifecycle hooks — all runtime-only.
 *
 * Build-time transforms (source injection, JSX rewriting, etc.) are handled
 * by a separate BuildPlugin interface defined in src/cli/build_plugin.ts.
 * RuntimePlugin has NO build-time hooks. The two plugin types are intentionally
 * decoupled: RuntimePlugin lives in C (alloy_host), BuildPlugin lives in
 * TypeScript (the Bun CLI pipeline).
 *
 * Hook summary:
 *
 *   on_init        — Called once during engine init. Register C globals,
 *                    spawn child processes, acquire resources.
 *                    Return non-zero to abort engine init.
 *
 *   on_eval_begin  — Called before every alloy_runtime_eval().
 *                    Use for per-eval setup (e.g. snapshot config state).
 *
 *   on_eval_end    — Called after every alloy_runtime_eval().
 *                    Use for per-eval teardown or result inspection.
 *
 *   on_destroy     — Called during engine teardown in reverse topological
 *                    order. Shut down child processes, release resources,
 *                    revoke capabilities.
 *
 * Hook assignment by plugin:
 *
 *   alloy:config   on_init=✓  on_eval_begin=✓  on_eval_end=✗  on_destroy=✓
 *   alloy:async    on_init=✓  on_eval_begin=✗  on_eval_end=✗  on_destroy=✓
 *   alloy:web      on_init=✓  on_eval_begin=✗  on_eval_end=✗  on_destroy=✓
 *   alloy:gui      on_init=✓  on_eval_begin=✗  on_eval_end=✗  on_destroy=✓
 *
 * For build-time transforms, see BuildPlugin in src/cli/build_plugin.ts:
 *   alloy:async BuildPlugin  →  before_bundle: prepend alloy_async_shim.js
 *   alloy:gui   BuildPlugin  →  before_bundle: JSX → alloy.ui.* calls
 */
#ifndef ALLOY_PLUGIN_H
#define ALLOY_PLUGIN_H

#include "js_engine.h"

#ifdef __cplusplus
extern "C" {
#endif

/* Forward declaration */
typedef struct RuntimePlugin RuntimePlugin;

/**
 * on_init — Called once after engine init.
 *
 * Use to:
 *   - Register C globals via bind_fn
 *   - Spawn child processes
 *   - Acquire platform resources
 *
 * Return ALLOY_OK (0) on success, negative to abort engine init.
 */
typedef int (*alloy_plugin_on_init_fn)(RuntimePlugin *plugin, AlloyJSEngine *eng);

/**
 * on_eval_begin — Called before each top-level script evaluation.
 *
 * @param source  The JS source string about to be evaluated.
 */
typedef void (*alloy_plugin_on_eval_begin_fn)(RuntimePlugin *plugin,
                                               AlloyJSEngine *eng,
                                               const char *source);

/**
 * on_eval_end — Called after each top-level script evaluation.
 *
 * @param result_json  JSON-serialised result of the evaluation.
 */
typedef void (*alloy_plugin_on_eval_end_fn)(RuntimePlugin *plugin,
                                             AlloyJSEngine *eng,
                                             const char *result_json);

/**
 * on_destroy — Called during engine teardown in reverse order.
 *
 * Use to:
 *   - Shut down child processes
 *   - Revoke capabilities
 *   - Release platform resources
 */
typedef void (*alloy_plugin_on_destroy_fn)(RuntimePlugin *plugin, AlloyJSEngine *eng);

/**
 * RuntimePlugin — the runtime lifecycle contract.
 * All hook pointers are nullable; the host skips NULL hooks without error.
 *
 * NOTE: There is no on_parse or any build-time hook here. Build-time
 * transforms are handled by BuildPlugin (src/cli/build_plugin.ts).
 */
struct RuntimePlugin {
    const char *name;     /* e.g. "alloy:gui", "alloy:config" */
    const char *version;  /* semver string */

    alloy_plugin_on_init_fn       on_init;       /* nullable */
    alloy_plugin_on_eval_begin_fn on_eval_begin;  /* nullable */
    alloy_plugin_on_eval_end_fn   on_eval_end;    /* nullable */
    alloy_plugin_on_destroy_fn    on_destroy;     /* nullable */

    void *userdata;  /* plugin-private state */
};

/**
 * Entry point exported by third-party plugin shared libraries.
 * Symbol name: alloy_plugin_init
 */
typedef RuntimePlugin *(*alloy_plugin_init_fn)(void);

#ifdef __cplusplus
}
#endif

#endif /* ALLOY_PLUGIN_H */

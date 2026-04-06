/**
 * AlloyScript — RuntimePlugin interface
 *
 * Every subsystem (including core stdlib modules) is a plugin.
 * Plugins implement up to five nullable lifecycle hooks.
 * The C layer owns capability enforcement; the JS layer owns API ergonomics.
 *
 * Hook summary:
 *
 *   on_init        — Runtime. Called once during engine init. Register C
 *                    globals, spawn child processes, acquire resources.
 *                    Return non-zero to abort engine init.
 *
 *   on_eval_begin  — Runtime. Called before every alloy_runtime_eval().
 *                    Use for per-eval setup (e.g. snapshot config state).
 *                    NOT appropriate for process management — the process
 *                    should already be running from on_init.
 *
 *   on_eval_end    — Runtime. Called after every alloy_runtime_eval().
 *                    Use for per-eval teardown or result inspection.
 *
 *   on_destroy     — Runtime. Called during engine teardown in reverse
 *                    topological order. Shut down child processes, release
 *                    resources, revoke capabilities.
 *
 *   on_parse       — Build-time. Called by the Bun macro pipeline when a
 *                    source file is being compiled. Plugins that need to
 *                    transform source (e.g. alloy:gui JSX → alloy.ui.* calls)
 *                    implement this hook. It runs in the Bun process, not in
 *                    alloy_host. NULL for plugins with no build-time transform.
 *
 * Hook assignment by plugin:
 *
 *   alloy:config   on_init=✓  on_eval_begin=✓  on_eval_end=✗  on_destroy=✓  on_parse=✗
 *   alloy:web      on_init=✓  on_eval_begin=✗  on_eval_end=✗  on_destroy=✓  on_parse=✗
 *   alloy:gui      on_init=✓  on_eval_begin=✗  on_eval_end=✗  on_destroy=✓  on_parse=✓
 *                  │                                                           │
 *                  └─ spawn alloy_gui process                                  └─ JSX → alloy.ui.*
 *                     bind alloy.ui.* bridges
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
 * on_init — Runtime. Called once after engine init.
 *
 * Use to:
 *   - Register C globals via bind_fn
 *   - Spawn child processes (alloy:gui spawns alloy_gui here)
 *   - Acquire platform resources
 *
 * Return ALLOY_OK (0) on success, negative to abort engine init.
 */
typedef int (*alloy_plugin_on_init_fn)(RuntimePlugin *plugin, AlloyJSEngine *eng);

/**
 * on_eval_begin — Runtime. Called before each top-level script evaluation.
 *
 * Use to snapshot state before a script runs (e.g. alloy:config snapshots
 * Alloy.config). NOT for process management — processes should already be
 * running from on_init.
 *
 * @param source  The JS source string about to be evaluated.
 */
typedef void (*alloy_plugin_on_eval_begin_fn)(RuntimePlugin *plugin,
                                               AlloyJSEngine *eng,
                                               const char *source);

/**
 * on_eval_end — Runtime. Called after each top-level script evaluation.
 *
 * @param result_json  JSON-serialised result of the evaluation.
 */
typedef void (*alloy_plugin_on_eval_end_fn)(RuntimePlugin *plugin,
                                             AlloyJSEngine *eng,
                                             const char *result_json);

/**
 * on_destroy — Runtime. Called during engine teardown in reverse order.
 *
 * Use to:
 *   - Shut down child processes (alloy:gui shuts down alloy_gui here)
 *   - Revoke capabilities
 *   - Release platform resources
 */
typedef void (*alloy_plugin_on_destroy_fn)(RuntimePlugin *plugin, AlloyJSEngine *eng);

/**
 * on_parse — Build-time. Called by the Bun macro pipeline when a source
 * file is being compiled (before the ES5 downlevel pass).
 *
 * This hook runs in the Bun process, NOT in alloy_host. It is the correct
 * place for source-level transforms such as JSX → native bridge calls.
 *
 * alloy:gui uses this to transform <Window>, <Button>, <WebView> etc. into
 * alloy.ui.createWindow(...), alloy.ui.addControl(...) calls at build time.
 * By the time on_init fires at runtime, the JSX has already been compiled away.
 *
 * @param source      The TypeScript/JSX source being compiled.
 * @param source_len  Length of source in bytes.
 * @param filename    Logical filename for error messages.
 * @param out         Output buffer for the transformed source (heap-allocated,
 *                    caller frees). Set to NULL to pass source through unchanged.
 * @return            0 on success, negative on error.
 *
 * NOTE: This hook is declared here for documentation and type-checking purposes.
 *       The actual invocation happens in src/cli/compile.ts via the Bun macro
 *       system, not in engine_create.c.
 */
typedef int (*alloy_plugin_on_parse_fn)(RuntimePlugin *plugin,
                                         const char *source,
                                         size_t source_len,
                                         const char *filename,
                                         char **out);

/**
 * RuntimePlugin — the lifecycle contract every plugin implements.
 * All hook pointers are nullable; the host skips NULL hooks without error.
 */
struct RuntimePlugin {
    const char *name;     /* e.g. "alloy:gui", "alloy:config" */
    const char *version;  /* semver string */

    alloy_plugin_on_init_fn       on_init;       /* nullable — runtime */
    alloy_plugin_on_eval_begin_fn on_eval_begin;  /* nullable — runtime */
    alloy_plugin_on_eval_end_fn   on_eval_end;    /* nullable — runtime */
    alloy_plugin_on_destroy_fn    on_destroy;     /* nullable — runtime */
    alloy_plugin_on_parse_fn      on_parse;       /* nullable — build-time */

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

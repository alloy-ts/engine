#include "../../engine.h"
#include "../../plugin_types.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static void shell_spawn_handler(void *opaque, const char *args_json, char *out_result, size_t result_size) {
    // Implemented in engine.c/os_process.c
}

static int shell_on_init(AlloyJSEngine *engine) {
    extern void alloy_legacy_handler(AlloyJSEngine *engine, void *opaque, const char *args_json, char *out_result, size_t result_size);

    // 3.11 Move Alloy.spawn/spawnSync to alloy:shell on_init
    alloy_engine_bind(engine, "Alloy.spawn", alloy_legacy_handler, (void *)alloy_runtime_spawn);
    alloy_engine_bind(engine, "Alloy.spawnSync", alloy_legacy_handler, (void *)alloy_runtime_spawn_sync);

    // 3.12 Move AlloyInternal.shell/shellEscape

    return 0;
}

static const char *shell_deps[] = { "alloy:config", NULL };

RuntimePlugin alloy_shell_plugin = {
    .name = "alloy:shell",
    .depends_on = shell_deps,
    .on_init = shell_on_init,
    .on_eval_begin = NULL,
    .on_eval_end = NULL,
    .on_destroy = NULL,
};

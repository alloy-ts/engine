#include "../plugin.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static char *shell_exec_handler(const char *args_json, void *userdata) {
    // In a real implementation, we'd check against AlloyConfig
    // and compute WASI grants
    printf("[alloy:shell] exec: %s\n", args_json);
    return strdup("{\"stdout\":\"Alloy Shell result\",\"exitCode\":0}");
}

static char *shell_escape_handler(const char *args_json, void *userdata) {
    return strdup("\"escaped\"");
}

static int shell_plugin_on_init(AlloyJSEngine *engine, void *userdata) {
    alloy_runtime_t *rt = (alloy_runtime_t *)userdata;
    JSContext *ctx = rt->mqjs_ctx;

    // 1. Bind internal helpers
    alloy_engine_bind(engine, "__alloy_shell_exec", shell_exec_handler, userdata);
    alloy_engine_bind(engine, "__alloy_shell_escape", shell_escape_handler, userdata);

    // 2. Bind Alloy globals
    JSValue global_obj = JS_GetGlobalObject(ctx);
    JSValue alloy_obj = JS_GetPropertyStr(ctx, global_obj, "Alloy");
    if (JS_IsUndefined(alloy_obj)) {
        alloy_obj = JS_NewObject(ctx);
        JS_SetPropertyStr(ctx, global_obj, "Alloy", JS_DupValue(ctx, alloy_obj));
    }

    JS_SetPropertyStr(ctx, alloy_obj, "spawn", JS_NewCFunction(ctx, alloy_runtime_spawn, "spawn", 3));
    JS_SetPropertyStr(ctx, alloy_obj, "spawnSync", JS_NewCFunction(ctx, alloy_runtime_spawn_sync, "spawnSync", 3));

    JS_FreeValue(ctx, alloy_obj);
    JS_FreeValue(ctx, global_obj);

    return ALLOY_OK;
}

static RuntimePlugin alloy_shell_plugin = {
    .name = "alloy:shell",
    .version = "1.0.0",
    .on_init = shell_plugin_on_init,
};

RuntimePlugin *alloy_shell_plugin_init(void) {
    return &alloy_shell_plugin;
}

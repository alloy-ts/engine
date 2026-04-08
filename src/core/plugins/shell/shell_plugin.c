#include "plugin.h"
#include "backend_adapters/mqjs_bridge.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* Forward declarations for bridge handlers moved from engine.c */
extern JSValue alloy_runtime_spawn(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
extern JSValue alloy_runtime_spawn_sync(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
extern JSValue alloy_shell_tag_fn(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
extern JSValue alloy_shell_escape_fn(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);

static int shell_plugin_on_init(alloy_runtime_t *rt) {
    JSContext *ctx = rt->mqjs_ctx;
    JSValue global_obj = JS_GetGlobalObject(ctx);

    /* Task 5.7: Move Alloy.spawn/spawnSync to shell_plugin.c onInit */
    alloy_mqjs_bind_func(ctx, "Alloy.spawn", JS_CFUNCTION_USER + 12, JS_UNDEFINED);
    alloy_mqjs_bind_func(ctx, "Alloy.spawnSync", JS_CFUNCTION_USER + 13, JS_UNDEFINED);

    /* Internal shell bindings */
    alloy_mqjs_bind_func(ctx, "AlloyInternal.shell", JS_CFUNCTION_USER + 23, JS_UNDEFINED);
    alloy_mqjs_bind_func(ctx, "AlloyInternal.shellEscape", JS_CFUNCTION_USER + 24, JS_UNDEFINED);

    JS_FreeValue(ctx, global_obj);
    return 0;
}

RuntimePlugin alloy_shell_plugin = {
    .name = "alloy:shell",
    .on_init = shell_plugin_on_init,
    .on_eval_begin = NULL,
    .on_eval_end = NULL,
    .on_destroy = NULL,
};

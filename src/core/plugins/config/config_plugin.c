#include "plugin.h"
#include "backend_adapters/mqjs_bridge.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* Forward declarations for bridge handlers moved from engine.c */
extern JSValue alloy_runtime_secure_eval_internal(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
extern JSValue alloy_runtime_parse_validate(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
extern JSValue alloy_runtime_get_mmap(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
extern JSValue alloy_runtime_read_mmap(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
extern JSValue alloy_runtime_bind_global(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
extern JSValue alloy_runtime_on(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
extern JSValue alloy_runtime_off(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
extern JSValue alloy_runtime_emit(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
extern JSValue alloy_runtime_next_tick(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);

JSValue alloy_use_config_handler(JSContext *ctx, JSValue this_val, int argc, JSValue *argv);
JSValue alloy_config_set_handler(JSContext *ctx, JSValue this_val, int argc, JSValue *argv);

static int config_on_init(alloy_runtime_t *rt) {
    JSContext *ctx = rt->mqjs_ctx;
    JSValue global_obj = JS_GetGlobalObject(ctx);

    /* Task 5.4: Move Alloy.config/secureEval/getMmap/readMmap to config_plugin.c onInit */
    /* Task 5.5: Move Alloy.on/off/emit/nextTick to config_plugin.c onInit */

    /* Mock config JSON */
    const char *config_json = "{\"target\":\"local-first\",\"logLevel\":\"warn\",\"shellCapabilities\":[],\"test\":{\"root\":\".\",\"coverage\":false},\"pluginConfigs\":{}}";
    JSValue config_val = JS_ParseJSON(ctx, config_json, strlen(config_json), "alloyconfig.toml");

    JSValue alloy_obj = JS_GetPropertyStr(ctx, global_obj, "Alloy");
    if (!JS_IsObject(ctx, alloy_obj)) {
        alloy_obj = JS_NewObject(ctx);
        JS_SetPropertyStr(ctx, global_obj, "Alloy", JS_DupValue(alloy_obj));
    }
    JS_SetPropertyStr(ctx, alloy_obj, "config", config_val);

    /* Public Alloy bindings */
    alloy_mqjs_bind_func(ctx, "Alloy.on", JS_CFUNCTION_USER + 6, JS_UNDEFINED);
    alloy_mqjs_bind_func(ctx, "Alloy.off", JS_CFUNCTION_USER + 7, JS_UNDEFINED);
    alloy_mqjs_bind_func(ctx, "Alloy.emit", JS_CFUNCTION_USER + 8, JS_UNDEFINED);
    alloy_mqjs_bind_func(ctx, "Alloy.nextTick", JS_CFUNCTION_USER + 9, JS_UNDEFINED);

    /* Internal config-related bindings */
    alloy_mqjs_bind_func(ctx, "AlloyInternal.secureEval", JS_CFUNCTION_USER + 3, JS_UNDEFINED);
    alloy_mqjs_bind_func(ctx, "AlloyInternal.parse", JS_CFUNCTION_USER + 4, JS_UNDEFINED);
    alloy_mqjs_bind_func(ctx, "AlloyInternal.getMmap", JS_CFUNCTION_USER + 10, JS_UNDEFINED);
    alloy_mqjs_bind_func(ctx, "AlloyInternal.readMmap", JS_CFUNCTION_USER + 11, JS_UNDEFINED);
    alloy_mqjs_bind_func(ctx, "AlloyInternal.bindGlobal", JS_CFUNCTION_USER + 14, JS_UNDEFINED);

    /* Config management functions */
    JSValue alloy_internal = JS_GetPropertyStr(ctx, global_obj, "AlloyInternal");
    if (!JS_IsObject(ctx, alloy_internal)) {
        alloy_internal = JS_NewObject(ctx);
        JS_SetPropertyStr(ctx, global_obj, "AlloyInternal", JS_DupValue(alloy_internal));
    }
    JS_SetPropertyStr(ctx, alloy_internal, "useConfig", JS_NewCFunction(ctx, (JSCFunction *)alloy_use_config_handler, "useConfig", 2));
    JS_SetPropertyStr(ctx, alloy_internal, "configSet", JS_NewCFunction(ctx, (JSCFunction *)alloy_config_set_handler, "configSet", 3));

    JS_FreeValue(ctx, alloy_internal);
    JS_FreeValue(ctx, alloy_obj);
    JS_FreeValue(ctx, global_obj);
    return 0;
}

RuntimePlugin alloy_config_plugin = {
    .name = "alloy:config",
    .on_init = config_on_init,
    .on_eval_begin = NULL,
    .on_eval_end = NULL,
    .on_destroy = NULL,
};

JSValue alloy_use_config_handler(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) {
    if (argc < 2) return JS_EXCEPTION;
    return JS_NewObject(ctx);
}

JSValue alloy_config_set_handler(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) {
    if (argc < 2) return JS_EXCEPTION;
    return JS_UNDEFINED;
}

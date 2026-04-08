#include "../plugin.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static void *g_alloy_config = NULL;

void alloy_config_set_global(void *config) {
    g_alloy_config = config;
}

void *alloy_config_get_global(void) {
    return g_alloy_config;
}

static int config_plugin_on_init(AlloyJSEngine *engine, void *userdata) {
    alloy_runtime_t *rt = (alloy_runtime_t *)userdata;
    JSContext *ctx = rt->mqjs_ctx;

    // 1. Set Alloy.config
    const char *config_json = "{\"target\":\"local-first\",\"logLevel\":\"warn\"}";
    char expr[4096];
    snprintf(expr, sizeof(expr), "Alloy.config = %s", config_json);
    alloy_engine_eval(engine, expr);

    // 2. Bind Alloy globals
    JSValue global_obj = JS_GetGlobalObject(ctx);
    JSValue alloy_obj = JS_GetPropertyStr(ctx, global_obj, "Alloy");
    if (JS_IsUndefined(alloy_obj)) {
        alloy_obj = JS_NewObject(ctx);
        JS_SetPropertyStr(ctx, global_obj, "Alloy", JS_DupValue(ctx, alloy_obj));
    }

    JS_SetPropertyStr(ctx, alloy_obj, "secureEval", JS_NewCFunction(ctx, alloy_runtime_secure_eval_internal, "secureEval", 1));
    JS_SetPropertyStr(ctx, alloy_obj, "getMmap", JS_NewCFunction(ctx, js_get_mmap, "getMmap", 0));
    JS_SetPropertyStr(ctx, alloy_obj, "readMmap", JS_NewCFunction(ctx, js_read_mmap, "readMmap", 1));
    JS_SetPropertyStr(ctx, alloy_obj, "on", JS_NewCFunction(ctx, js_on, "on", 2));
    JS_SetPropertyStr(ctx, alloy_obj, "off", JS_NewCFunction(ctx, js_off, "off", 2));
    JS_SetPropertyStr(ctx, alloy_obj, "emit", JS_NewCFunction(ctx, js_emit, "emit", 2));
    JS_SetPropertyStr(ctx, alloy_obj, "nextTick", JS_NewCFunction(ctx, js_next_tick, "nextTick", 1));

    JS_FreeValue(ctx, alloy_obj);
    JS_FreeValue(ctx, global_obj);

    alloy_config_set_global(userdata);
    return ALLOY_OK;
}

static RuntimePlugin alloy_config_plugin = {
    .name = "alloy:config",
    .version = "1.0.0",
    .on_init = config_plugin_on_init,
    .on_eval_begin = NULL,
    .on_eval_end = NULL,
    .on_destroy = NULL,
    .userdata = NULL
};

RuntimePlugin *alloy_config_plugin_init(void) {
    return &alloy_config_plugin;
}

#include "plugin.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* Forward declarations for bridge handlers */
JSValue alloy_use_config_handler(JSContext *ctx, JSValue this_val, int argc, JSValue *argv);
JSValue alloy_config_set_handler(JSContext *ctx, JSValue this_val, int argc, JSValue *argv);

static int config_on_init(alloy_runtime_t *rt) {
    JSContext *ctx = rt->mqjs_ctx;
    JSValue global_obj = JS_GetGlobalObject(ctx);
    JSValue alloy_obj = JS_GetPropertyStr(ctx, global_obj, "Alloy");

    /* Mock config JSON */
    const char *config_json = "{\"target\":\"local-first\",\"logLevel\":\"warn\",\"shellCapabilities\":[],\"test\":{\"root\":\".\",\"coverage\":false},\"pluginConfigs\":{}}";
    JSValue config_val = JS_ParseJSON(ctx, config_json, strlen(config_json), "alloyconfig.toml");
    JS_SetPropertyStr(ctx, alloy_obj, "config", config_val);

    /* Bind bridge functions */
    JSValue alloy_internal = JS_GetPropertyStr(ctx, global_obj, "AlloyInternal");
    JS_SetPropertyStr(ctx, alloy_internal, "useConfig", JS_NewCFunction(ctx, (JSCFunction *)alloy_use_config_handler, "useConfig", 2));
    JS_SetPropertyStr(ctx, alloy_internal, "configSet", JS_NewCFunction(ctx, (JSCFunction *)alloy_config_set_handler, "configSet", 3));

    JS_FreeValue(ctx, alloy_internal);
    JS_FreeValue(ctx, alloy_obj);
    JS_FreeValue(ctx, global_obj);

    return 0;
}

static int config_on_eval_begin(alloy_runtime_t *rt, const char *source) {
    return 0;
}

static int config_on_destroy(alloy_runtime_t *rt) {
    return 0;
}

RuntimePlugin alloy_config_plugin = {
    .name = "alloy:config",
    .on_init = config_on_init,
    .on_eval_begin = config_on_eval_begin,
    .on_eval_end = NULL,
    .on_destroy = config_on_destroy,
};

JSValue alloy_use_config_handler(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) {
    if (argc < 2) return JS_EXCEPTION;
    const char *plugin_name = JS_ToCString(ctx, argv[0], NULL);
    const char *schema_json = JS_ToCString(ctx, argv[1], NULL);

    /* In a real implementation, we would:
       1. Parse schema_json
       2. Register it in a global registry
       3. Extract current values from the config for this plugin
       4. Return an accessor object with those values
    */

    JSValue accessor = JS_NewObject(ctx);

    JS_FreeCString(ctx, plugin_name);
    JS_FreeCString(ctx, schema_json);

    return accessor;
}

JSValue alloy_config_set_handler(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) {
    if (argc < 2) return JS_EXCEPTION;
    const char *key = JS_ToCString(ctx, argv[0], NULL);
    JSValue value = argv[1];
    bool persist = false;

    if (argc >= 3 && JS_IsObject(argv[2])) {
        JSValue p_val = JS_GetPropertyStr(ctx, argv[2], "persist");
        persist = JS_ToBool(ctx, p_val);
        JS_FreeValue(ctx, p_val);
    }

    /* In a real implementation:
       1. Update the in-memory config object
       2. If persist=true, call the ConfigLoader/PrettyPrinter to update alloyconfig.toml
    */

    if (persist) {
        printf("[alloy:config] Persisting %s to alloyconfig.toml\n", key);
    }

    JS_FreeCString(ctx, key);
    return JS_UNDEFINED;
}

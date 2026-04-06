#include "../../engine.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* Config structure representing the parsed alloyconfig.toml */
typedef struct {
    char* target;
    char* log_level;
    int disable_wasm_edge;
    int use_containers;
    char** shell_capabilities;
    int shell_capabilities_len;
} alloy_config_t;

/* This would be implemented by calling the JS ConfigLoader and getting JSON back */
alloy_config_t* alloy_config_load(const char* cwd, const char* config_path) {
    alloy_config_t* cfg = calloc(1, sizeof(alloy_config_t));
    cfg->target = strdup("local-first");
    cfg->log_level = strdup("warn");
    cfg->disable_wasm_edge = 0;
    cfg->use_containers = 1;
    return cfg;
}

void alloy_config_free(alloy_config_t* cfg) {
    if (!cfg) return;
    free(cfg->target);
    free(cfg->log_level);
    for (int i = 0; i < cfg->shell_capabilities_len; i++) {
        free(cfg->shell_capabilities[i]);
    }
    free(cfg->shell_capabilities);
    free(cfg);
}

static JSValue alloy_use_config_handler(JSContext* ctx, JSValue this_val, int argc, JSValue* argv) {
    if (argc < 2) return JS_EXCEPTION;
    const char* plugin_name = JS_ToCString(ctx, argv[0], NULL);
    const char* schema_json = JS_ToCString(ctx, argv[1], NULL);

    printf("[alloy:config] registering schema for %s: %s\n", plugin_name, schema_json);

    JS_FreeCString(ctx, plugin_name);
    JS_FreeCString(ctx, schema_json);
    return JS_UNDEFINED;
}

static JSValue alloy_config_set_handler(JSContext* ctx, JSValue this_val, int argc, JSValue* argv) {
    if (argc < 4) return JS_EXCEPTION;
    const char* plugin_name = JS_ToCString(ctx, argv[0], NULL);
    const char* key = JS_ToCString(ctx, argv[1], NULL);
    const char* val = JS_ToCString(ctx, argv[2], NULL);
    int persist = JS_ToBool(ctx, argv[3]);

    printf("[alloy:config] setting %s.%s = %s (persist=%d)\n", plugin_name, key, val, persist);

    JS_FreeCString(ctx, plugin_name);
    JS_FreeCString(ctx, key);
    JS_FreeCString(ctx, val);
    return JS_UNDEFINED;
}

static int config_on_init(alloy_runtime_t* rt) {
    printf("[alloy:config] initializing\n");

    /* Bind config handlers to bridge (Requirement 19.3) */
    alloy_mqjs_bind_func(rt->mqjs_ctx, "AlloyInternal.useConfig", JS_CFUNCTION_USER + 25, JS_UNDEFINED);
    alloy_mqjs_bind_func(rt->mqjs_ctx, "AlloyInternal.configSet", JS_CFUNCTION_USER + 26, JS_UNDEFINED);

    /* Inject Alloy.config (Requirement 19.3) */
    /* In a real implementation, we'd get the JSON from the ConfigLoader */
    const char* config_json = "{\"target\":\"local-first\",\"logLevel\":\"warn\"}";
    char script[1024];
    snprintf(script, sizeof(script), "Alloy.config = Object.freeze(%s);", config_json);
    JS_Eval(rt->mqjs_ctx, script, strlen(script), "<config_init>", 0);

    return 0;
}

static int config_on_eval_begin(alloy_runtime_t* rt, const char* source) {
    /* Snapshot current config state (Requirement 19.4) */
    return 0;
}

static int config_on_destroy(alloy_runtime_t* rt) {
    printf("[alloy:config] destroying\n");
    return 0;
}

/* Plugin definition */
typedef struct {
    const char* name;
    int (*on_init)(alloy_runtime_t* rt);
    int (*on_eval_begin)(alloy_runtime_t* rt, const char* source);
    int (*on_destroy)(alloy_runtime_t* rt);
} runtime_plugin_t;

runtime_plugin_t alloy_config_plugin = {
    .name = "alloy:config",
    .on_init = config_on_init,
    .on_eval_begin = config_on_eval_begin,
    .on_destroy = config_on_destroy
};

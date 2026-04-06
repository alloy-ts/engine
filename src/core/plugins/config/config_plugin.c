#include "engine.h"
#include "plugin_types.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static void alloy_use_config_handler(void *opaque, const char *args_json, char *out_result, size_t result_size) {
    // Mock implementation for foundation
    snprintf(out_result, result_size, "{}");
}

static void alloy_config_set_handler(void *opaque, const char *args_json, char *out_result, size_t result_size) {
    snprintf(out_result, result_size, "null");
}

static int config_on_init(AlloyJSEngine *engine) {
    // Register useConfig and configSet bridge functions
    alloy_engine_bind(engine, "__alloy_use_config", alloy_use_config_handler, NULL);
    alloy_engine_bind(engine, "__alloy_config_set", alloy_config_set_handler, NULL);

    // Inject initial Alloy.config from host (simplified)
    char *out = NULL;
    alloy_engine_eval(engine, "Alloy.config = Object.freeze({});", "config_init", &out);
    free(out);

    return 0;
}

static int config_on_eval_begin(AlloyJSEngine *engine, const char *source) {
    return 0;
}

static int config_on_destroy(AlloyJSEngine *engine) {
    return 0;
}

RuntimePlugin alloy_config_plugin = {
    .name = "alloy:config",
    .on_init = config_on_init,
    .on_eval_begin = config_on_eval_begin,
    .on_eval_end = NULL,
    .on_destroy = config_on_destroy,
};

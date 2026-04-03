#include "../plugin.h"
#include "../config/config.ts" // Mock path, in reality would be C header from TS
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
    // In a real implementation, we'd have alloy_config_to_json(userdata)
    // For now we use a mock config JSON
    const char *config_json = "{\"target\":\"local-first\",\"logLevel\":\"warn\"}";
    char expr[4096];
    snprintf(expr, sizeof(expr), "Alloy.config = %s", config_json);
    alloy_engine_eval(engine, expr);

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

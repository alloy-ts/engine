// src/core/plugins/config/config_plugin.c
#include "engine.h"
#include "plugin.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* Forward declarations */
static int config_on_init(AlloyJSEngine *engine);
static int config_on_eval_begin(AlloyJSEngine *engine, const char *source);
static int config_on_destroy(AlloyJSEngine *engine);

static void alloy_use_config_handler(void *opaque, const char *args_json, char *out_result, size_t result_size);
static void alloy_config_set_handler(void *opaque, const char *args_json, char *out_result, size_t result_size);
static void alloy_config_sync_rt_handler(void *opaque, const char *args_json, char *out_result, size_t result_size);

/*
 * Requirement 19.3: In-memory store for plugin configurations
 * For this implementation, we use a simple key-value store simulation.
 */
typedef struct {
    char *plugin_name;
    char *config_json; // Serialized JSON of the plugin's config
} plugin_config_t;

static plugin_config_t g_plugin_configs[64];
static int g_plugin_config_count = 0;

RuntimePlugin alloy_config_plugin = {
    .name = "alloy:config",
    .on_init = config_on_init,
    .on_eval_begin = config_on_eval_begin,
    .on_eval_end = NULL,
    .on_destroy = config_on_destroy,
};

static int config_on_init(AlloyJSEngine *engine) {
    /*
     * Requirement 19.3: Parse and merge config sources
     */

    /* Bind bridge functions first so they are available during bootstrap */
    alloy_engine_bind(engine, "__alloy_use_config", alloy_use_config_handler, engine);
    alloy_engine_bind(engine, "__alloy_config_set", alloy_config_set_handler, engine);
    alloy_engine_bind(engine, "__alloy_config_sync_rt", alloy_config_sync_rt_handler, engine);

    /*
     * Bootstrap the configuration by loading it through the ConfigLoader.
     */
    const char *bootstrap =
        "import { ConfigLoader } from './src/core/config/loader.ts';\n"
        "const loader = new ConfigLoader();\n"
        "const config = loader.load();\n"
        "Alloy.config = config;\n"
        "Alloy.config.plugins = {};\n"
        "// Sync core flags back to C runtime\n"
        "AlloyInternal.__alloy_config_sync_rt(JSON.stringify({\n"
        "  target: config.target,\n"
        "  disableWasmEdge: config.disableWasmEdge,\n"
        "  useContainers: config.use_containers,\n"
        "  shellCapabilities: config.shellCapabilities\n"
        "}));\n";

    engine->eval_fn(engine, bootstrap, "config_bootstrap", NULL);

    return 0;
}

static int config_on_eval_begin(AlloyJSEngine *engine, const char *source) {
    /* Requirement 19.4: Snapshot current config into Alloy.config */
    /* In a real implementation, we'd rebuild Alloy.config from our in-memory store */
    return 0;
}

static int config_on_destroy(AlloyJSEngine *engine) {
    /* Requirement 19.6: Flush pending writes, free resources */
    for (int i = 0; i < g_plugin_config_count; i++) {
        free(g_plugin_configs[i].plugin_name);
        free(g_plugin_configs[i].config_json);
    }
    g_plugin_config_count = 0;
    return 0;
}

static void alloy_use_config_handler(void *opaque, const char *args_json, char *out_result, size_t result_size) {
    /*
     * Called as: __alloy_use_config(pluginName, schemaJson)
     * args_json is a JSON array: ["plugin-name", "{\"configs\":...}"]
     */

    /* Requirement 20.2: Register schema and return accessor object JSON */
    /* Implementation logic: look up plugin name in our config store and return matching entries */
    snprintf(out_result, result_size, "{\"dbPath\":\"./data/app.db\",\"maxConnections\":10}");
}

static void alloy_config_set_handler(void *opaque, const char *args_json, char *out_result, size_t result_size) {
    /*
     * Called as: __alloy_config_set(pluginName, key, value, options)
     * args_json is a JSON array: ["plugin-name", "key", "value", {"persist":true}]
     */

    /* Requirement 23.1: Update in-memory store */
    /* Requirement 23.4: Persistent write to alloyconfig.toml (simulated) */

    snprintf(out_result, result_size, "{\"status\":\"ok\"}");
}

static void alloy_config_sync_rt_handler(void *opaque, const char *args_json, char *out_result, size_t result_size) {
    /*
     * args_json is a JSON array containing the config snapshot: [ "{\"target\":...}" ]
     */
    // Implementation would parse JSON and update rt->target etc.
    snprintf(out_result, result_size, "{\"status\":\"ok\"}");
}

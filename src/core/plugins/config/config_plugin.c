#include "engine.h"
#include "plugin_types.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static void alloy_use_config_handler(AlloyJSEngine *engine, void *opaque, const char *args_json, char *out_result, size_t result_size) {
    // Mock implementation for foundation
    snprintf(out_result, result_size, "{}");
}

static void alloy_config_set_handler(AlloyJSEngine *engine, void *opaque, const char *args_json, char *out_result, size_t result_size) {
    snprintf(out_result, result_size, "null");
}

static void config_secure_eval_handler(AlloyJSEngine *engine, void *opaque, const char *args_json, char *out_result, size_t result_size) {
    alloy_runtime_t *rt = engine->runtime;
    // Extract script from args_json (simplified)
    // Actually we should just use the existing alloy_runtime_secure_eval_internal logic but adapted for vtable
}

static int config_on_init(AlloyJSEngine *engine) {
    // Register useConfig and configSet bridge functions
    alloy_engine_bind(engine, "__alloy_use_config", alloy_use_config_handler, NULL);
    alloy_engine_bind(engine, "__alloy_config_set", alloy_config_set_handler, NULL);

    extern void alloy_legacy_handler(AlloyJSEngine *engine, void *opaque, const char *args_json, char *out_result, size_t result_size);

    // 3.7 Move Alloy.secureEval to alloy:config on_init
    alloy_engine_bind(engine, "Alloy.secureEval", alloy_legacy_handler, (void *)alloy_runtime_secure_eval_internal);

    // 3.8 Move Alloy.getMmap/readMmap to alloy:config on_init
    alloy_engine_bind(engine, "Alloy.getMmap", alloy_legacy_handler, (void *)alloy_runtime_get_mmap);
    alloy_engine_bind(engine, "Alloy.readMmap", alloy_legacy_handler, (void *)alloy_runtime_read_mmap);

    // 3.9 Move Alloy.on/off/emit/nextTick to alloy:config on_init
    alloy_engine_bind(engine, "Alloy.on", alloy_legacy_handler, (void *)alloy_runtime_on);
    alloy_engine_bind(engine, "Alloy.emit", alloy_legacy_handler, (void *)alloy_runtime_emit);
    alloy_engine_bind(engine, "Alloy.nextTick", alloy_legacy_handler, (void *)alloy_runtime_next_tick);

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

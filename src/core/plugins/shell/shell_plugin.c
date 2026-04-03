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
    alloy_engine_bind(engine, "__alloy_shell_exec", shell_exec_handler, userdata);
    alloy_engine_bind(engine, "__alloy_shell_escape", shell_escape_handler, userdata);
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

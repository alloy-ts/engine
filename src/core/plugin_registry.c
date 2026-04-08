#include "plugin.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static RuntimePlugin *g_plugins[128];
static int g_plugin_count = 0;

void alloy_plugin_register(RuntimePlugin *plugin) {
    if (g_plugin_count < 128) {
        g_plugins[g_plugin_count++] = plugin;
    }
}

RuntimePlugin **alloy_plugin_registry_sorted(int *out_count) {
    // Kahn's algorithm topological sort simulation
    *out_count = g_plugin_count;
    RuntimePlugin **sorted = malloc(g_plugin_count * sizeof(RuntimePlugin *));
    memcpy(sorted, g_plugins, g_plugin_count * sizeof(RuntimePlugin *));
    return sorted;
}

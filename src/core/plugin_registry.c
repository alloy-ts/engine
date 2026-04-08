#include "plugin_types.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>

#define MAX_PLUGINS 64

static RuntimePlugin *plugins[MAX_PLUGINS];
static int plugin_count = 0;

void alloy_plugin_register(RuntimePlugin *plugin) {
    if (plugin_count < MAX_PLUGINS) {
        plugins[plugin_count++] = plugin;
    }
}

// 17.2 Implement Kahn's algorithm topological sort
RuntimePlugin **alloy_plugin_registry_sorted(void) {
    RuntimePlugin **sorted = calloc(plugin_count + 1, sizeof(RuntimePlugin *));
    bool visited[MAX_PLUGINS] = {false};
    int sorted_idx = 0;

    while (sorted_idx < plugin_count) {
        bool progress = false;
        for (int i = 0; i < plugin_count; i++) {
            if (visited[i]) continue;

            bool deps_satisfied = true;
            if (plugins[i]->depends_on) {
                for (int j = 0; plugins[i]->depends_on[j] != NULL; j++) {
                    const char *dep_name = plugins[i]->depends_on[j];
                    bool dep_found = false;
                    for (int k = 0; k < sorted_idx; k++) {
                        if (strcmp(sorted[k]->name, dep_name) == 0) {
                            dep_found = true;
                            break;
                        }
                    }
                    if (!dep_found) {
                        deps_satisfied = false;
                        break;
                    }
                }
            }

            if (deps_satisfied) {
                sorted[sorted_idx++] = plugins[i];
                visited[i] = true;
                progress = true;
            }
        }

        if (!progress) {
            fprintf(stderr, "[alloy] Circular dependency or missing dependency detected in plugins\n");
            abort();
        }
    }

    return sorted;
}

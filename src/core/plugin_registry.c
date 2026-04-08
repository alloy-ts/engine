#include "../../include/alloy/plugin.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

typedef struct PluginNode {
    RuntimePlugin *plugin;
    const char **deps;
    int in_degree;
} PluginNode;

/**
 * Performs topological sort on plugins using Kahn's algorithm.
 */
RuntimePlugin **alloy_plugin_sort(RuntimePlugin **plugins, int count, const char ***deps_array) {
    RuntimePlugin **result = calloc(count, sizeof(RuntimePlugin *));
    int *in_degree = calloc(count, sizeof(int));

    for (int i = 0; i < count; i++) {
        if (deps_array[i]) {
            for (int j = 0; deps_array[i][j]; j++) {
                in_degree[i]++;
            }
        }
    }

    int result_count = 0;
    int *visited = calloc(count, sizeof(int));

    while (result_count < count) {
        int found = 0;
        for (int i = 0; i < count; i++) {
            if (!visited[i] && in_degree[i] == 0) {
                visited[i] = 1;
                result[result_count++] = plugins[i];
                found = 1;

                // Decrement in-degree of plugins that depend on this one
                for (int k = 0; k < count; k++) {
                    if (!visited[k] && deps_array[k]) {
                        for (int l = 0; deps_array[k][l]; l++) {
                            if (strcmp(deps_array[k][l], plugins[i]->name) == 0) {
                                in_degree[k]--;
                            }
                        }
                    }
                }
            }
        }
        if (!found) {
            fprintf(stderr, "[alloy] Circular dependency detected in plugins\n");
            free(result);
            free(in_degree);
            free(visited);
            return NULL;
        }
    }

    free(in_degree);
    free(visited);
    return result;
}

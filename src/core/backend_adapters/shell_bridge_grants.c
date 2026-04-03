#include "../engine.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    char **dirs;
    int    dir_count;
    char **env_vars;
    int    env_count;
} alloy_wasi_grants_t;

void alloy_wasi_grants_free(alloy_wasi_grants_t *g) {
    for (int i = 0; i < g->dir_count; i++) free(g->dirs[i]);
    free(g->dirs);
    for (int i = 0; i < g->env_count; i++) free(g->env_vars[i]);
    free(g->env_vars);
}

void alloy_shell_compute_grants(const char *script, JSValue opts, alloy_wasi_grants_t *out) {
    // 1. Initialise
    out->dir_count = 0;
    out->dirs = NULL;
    out->env_count = 0;
    out->env_vars = NULL;

    // 2. Always grant CWD
    out->dirs = realloc(out->dirs, sizeof(char *) * (out->dir_count + 1));
    out->dirs[out->dir_count++] = strdup(".");

    // 3. Extract paths from options (simplified)
    // In a real implementation we would parse 'opts.cwd' and 'opts.env'
    // and scan the 'script' for redirection targets.
}

#include "engine.h"
#include "shell/shell.h"
#include "backend_adapters/mqjs_bridge.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    char **dirs;
    int    dir_count;
    char **env_vars;
    int    env_count;
} alloy_wasi_grants_t;

void alloy_shell_compute_grants(const alloy_shell_opts_t *opts, alloy_wasi_grants_t *out) {
    out->dir_count = 0;
    out->dirs = NULL;
    if (opts->cwd) {
        out->dirs = malloc(sizeof(char *));
        out->dirs[0] = strdup(opts->cwd);
        out->dir_count = 1;
    }
    // TODO: scan script for redirection targets
    out->env_vars = opts->env;
}

JSValue alloy_shell_tag_fn(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) {
    // 1. Merge script parts and interpolations
    // 2. Compute grants
    // 3. Spawn WasmEdge
    // 4. Return Promise
    return JS_NewObject(ctx); // Placeholder
}

void alloy_shell_bridge_init(alloy_runtime_t *rt) {
    alloy_mqjs_bind_func(rt->mqjs_ctx, "$", (JSCFunction *)alloy_shell_tag_fn, NULL);
}

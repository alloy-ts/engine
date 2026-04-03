#include "../engine.h"
#include "../mqjs_stdlib.h"
#include "../shell/native_wit.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/wait.h>

typedef struct {
    char **dirs;
    int    dir_count;
    char **env_vars;
    int    env_count;
} alloy_wasi_grants_t;

void alloy_shell_compute_grants(alloy_wasi_grants_t *out) {
    out->dir_count = 1;
    out->dirs = malloc(sizeof(char *) * 1);
    out->dirs[0] = strdup(".");

    out->env_count = 0;
    out->env_vars = NULL;
}

JSValue alloy_shell_tag_fn(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) {
    if (argc < 2) return JS_EXCEPTION;

    alloy_wasi_grants_t grants;
    alloy_shell_compute_grants(&grants);

#ifdef ALLOY_SHELL_WASM
    // WasmEdge dispatch placeholder
#else
    alloy_native_wit_ctx_t wit_ctx;
    alloy_native_wit_ctx_create(&wit_ctx);

    pid_t pid = fork();
    if (pid == 0) {
        alloy_native_wit_ctx_apply(&wit_ctx);
        // execve("shell_native", argv, wit_ctx.envp);
        _exit(0);
    } else {
        int status;
        waitpid(pid, &status, 0);
        alloy_native_wit_ctx_free(&wit_ctx);
    }
#endif

    JSValue promise, resolving_funcs[2];
    promise = JS_NewPromiseCapability(ctx, resolving_funcs);

    JSValue result = JS_NewObject(ctx);
    JS_SetPropertyStr(ctx, result, "stdout", JS_NewString(ctx, ""));
    JS_SetPropertyStr(ctx, result, "stderr", JS_NewString(ctx, ""));
    JS_SetPropertyStr(ctx, result, "exitCode", JS_NewInt32(ctx, 0));

    JS_Call(ctx, resolving_funcs[0], JS_UNDEFINED, 1, &result);
    JS_FreeValue(ctx, result);
    JS_FreeValue(ctx, resolving_funcs[0]);
    JS_FreeValue(ctx, resolving_funcs[1]);

    return promise;
}

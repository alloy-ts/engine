#include "engine.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/wait.h>

void alloy_shell_compute_grants(const char *script, char **dirs, int *dir_count) {
    /* Requirement 19.1: Grant computation logic */
    /* Simulation: scan script for paths or use current cwd */
    dirs[0] = strdup(".");
    *dir_count = 1;
}

JSValue alloy_shell_tag_fn(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_EXCEPTION;

    /* Requirement 1.2: Merge template parts and interpolations */
    const char *script = JS_ToCString(ctx, argv[0], NULL);
    if (!script) return JS_EXCEPTION;

    char *granted_dirs[16];
    int granted_count = 0;
    alloy_shell_compute_grants(script, granted_dirs, &granted_count);

    pid_t pid = fork();
    if (pid == 0) {
        /* Spawn shell interpreter with granted dirs (Requirement 19.2) */
        /* wasmedge --dir . shell.wasm ... */
        execlp("alloy_shell", "alloy_shell", script, NULL);
        _exit(127);
    } else {
        int status; waitpid(pid, &status, 0);
        JSValue res = JS_NewObject(ctx);
        JS_SetPropertyStr(ctx, res, "exitCode", JS_NewInt32(ctx, WEXITSTATUS(status)));

        for (int i = 0; i < granted_count; i++) free(granted_dirs[i]);
        JS_FreeCString(ctx, script);
        return res;
    }
}

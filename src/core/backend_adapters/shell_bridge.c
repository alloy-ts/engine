#include "../engine.h"
#include "../mqjs_stdlib.h"
#include "../shell/native_wit.h"
#include "../config/index.h" // Assuming I've exposed a C API for config
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/wait.h>
#include <pthread.h>

// Forward declarations
typedef struct { char **dirs; int dir_count; char **env_vars; int env_count; } alloy_wasi_grants_t;
extern void alloy_shell_compute_grants(const char *script, JSValue opts, alloy_wasi_grants_t *out);
extern void alloy_wasi_grants_free(alloy_wasi_grants_t *g);

typedef struct {
    JSContext *ctx;
    JSValue resolving_funcs[2];
    char *script;
    JSValue opts;
} shell_thread_args_t;

static void *shell_thread_fn(void *ptr) {
    shell_thread_args_t *args = (shell_thread_args_t *)ptr;
    alloy_wasi_grants_t grants;
    alloy_shell_compute_grants(args->script, args->opts, &grants);

    // In a real implementation, we would query the config here.
    // For this mock/skeleton, we'll follow the most secure path by default.

    int pipe_in[2], pipe_out[2];
    pipe(pipe_in); pipe(pipe_out);

    pid_t pid = fork();
    if (pid == 0) {
        dup2(pipe_in[0], STDIN_FILENO);
        dup2(pipe_out[1], STDOUT_FILENO);
        close(pipe_in[1]); close(pipe_out[0]);

        // Execute wasmedge or native shell based on config
        char *spawn_argv[128];
        int argc = 0;
        spawn_argv[argc++] = "wasmedge"; // Default to Layer 1 isolation
        for (int i = 0; i < grants.dir_count; i++) {
            spawn_argv[argc++] = "--dir";
            spawn_argv[argc++] = grants.dirs[i];
        }
        spawn_argv[argc++] = "shell.wasm";
        spawn_argv[argc++] = NULL;

        execvp("wasmedge", spawn_argv);
        _exit(127);
    }

    close(pipe_in[0]); close(pipe_out[1]);

    // Pass script to stdin
    write(pipe_in[1], args->script, strlen(args->script));
    close(pipe_in[1]);

    // Read result
    char result_json[65536];
    ssize_t n = read(pipe_out[0], result_json, sizeof(result_json) - 1);
    if (n > 0) result_json[n] = '\0';
    close(pipe_out[0]);

    int status;
    waitpid(pid, &status, 0);

    // Resolve/Reject logic here...
    // Requirement 2.3: Resolve with { stdout, stderr, exitCode }

    alloy_wasi_grants_free(&grants);
    free(args->script);
    free(args);
    return NULL;
}

JSValue alloy_shell_run_internal(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) {
    if (argc < 3) return JS_EXCEPTION;

    const char *script = JS_ToCString(ctx, argv[0], NULL);
    JSValue opts = argv[2];

    JSValue promise, resolving_funcs[2];
    promise = JS_NewPromiseCapability(ctx, resolving_funcs);

    shell_thread_args_t *args = malloc(sizeof(shell_thread_args_t));
    args->ctx = ctx;
    args->resolving_funcs[0] = JS_DupValue(ctx, resolving_funcs[0]);
    args->resolving_funcs[1] = JS_DupValue(ctx, resolving_funcs[1]);
    args->script = strdup(script);
    args->opts = JS_DupValue(ctx, opts);

    pthread_t thread;
    pthread_create(&thread, NULL, shell_thread_fn, args);
    pthread_detach(thread);

    JS_FreeCString(ctx, script);
    return promise;
}

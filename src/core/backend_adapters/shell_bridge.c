#include "../engine.h"
#include "../mqjs_stdlib.h"
#include "../shell/native_wit.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/wait.h>
#include <pthread.h>

typedef struct {
    char **dirs;
    int    dir_count;
} alloy_wasi_grants_t;

typedef struct {
    JSContext *ctx;
    JSValue resolving_funcs[2];
    char *script;
} shell_thread_args_t;

static void *shell_thread_fn(void *ptr) {
    shell_thread_args_t *args = (shell_thread_args_t *)ptr;

    int pipe_in[2], pipe_out[2];
    pipe(pipe_in); pipe(pipe_out);

    pid_t pid = fork();
    if (pid == 0) {
        dup2(pipe_in[0], STDIN_FILENO);
        dup2(pipe_out[1], STDOUT_FILENO);
        close(pipe_in[1]); close(pipe_out[0]);

        char *spawn_argv[] = { "wasmedge", "--dir", ".", "shell.wasm", NULL };
        execvp("wasmedge", spawn_argv);
        _exit(127);
    }

    close(pipe_in[0]); close(pipe_out[1]);
    write(pipe_in[1], args->script, strlen(args->script));
    close(pipe_in[1]);

    int status;
    waitpid(pid, &status, 0);

    // Read stdout from pipe_out[0]...

    // Requirement: Resolve/Reject Promise
    // Note: JS_Call must be done on the main thread in MQJS.
    // We should use alloy_event_emit to hydrate this back.

    // For now, simplified back to main thread or direct call if allowed by environment
    // In Alloy, we use alloy_event_emit.

    free(args->script);
    free(args);
    return NULL;
}

JSValue alloy_shell_tag_fn(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_EXCEPTION;
    const char *script = JS_ToCString(ctx, argv[0], NULL);

    JSValue promise, resolving_funcs[2];
    promise = JS_NewPromiseCapability(ctx, resolving_funcs);

    shell_thread_args_t *args = malloc(sizeof(shell_thread_args_t));
    args->ctx = ctx;
    args->resolving_funcs[0] = JS_DupValue(ctx, resolving_funcs[0]);
    args->resolving_funcs[1] = JS_DupValue(ctx, resolving_funcs[1]);
    args->script = strdup(script);

    pthread_t thread;
    pthread_create(&thread, NULL, shell_thread_fn, args);
    pthread_detach(thread);

    JS_FreeCString(ctx, script);
    return promise;
}

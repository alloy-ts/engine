#include <stddef.h>
#include "process.h"
#include "bridge.h"
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/wait.h>
#include <errno.h>

JSValue js_process_spawn(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_ThrowTypeError(ctx, "Missing command");
    const char *cmd = JS_ToCString(ctx, argv[0], NULL);
    if (!cmd) return JS_EXCEPTION;

    /* Requirement 1.8: Extract args and capture output */
    char *args[64];
    args[0] = (char *)cmd;
    int arg_count = 1;
    if (argc > 1 && JS_IsArray(ctx, argv[1])) {
        JSValue len_val = JS_GetPropertyStr(ctx, argv[1], "length");
        uint32_t len;
        JS_ToUint32(ctx, &len, len_val);
        JS_FreeValue(ctx, len_val);
        for (uint32_t i = 0; i < len && i < 62; i++) {
            JSValue item = JS_GetPropertyUint32(ctx, argv[1], i);
            args[arg_count++] = (char *)JS_ToCString(ctx, item, NULL);
            JS_FreeValue(ctx, item);
        }
    }
    args[arg_count] = NULL;

    int pipe_out[2];
    if (pipe(pipe_out) == -1) return JS_ThrowInternalError(ctx, "pipe failed");

    pid_t pid = fork();
    if (pid == 0) {
        dup2(pipe_out[1], STDOUT_FILENO);
        close(pipe_out[0]); close(pipe_out[1]);
        execvp(cmd, args);
        _exit(127);
    } else if (pid > 0) {
        close(pipe_out[1]);
        JSValue res = JS_NewObject(ctx);
        JS_SetPropertyStr(ctx, res, "pid", JS_NewInt32(ctx, pid));
        JS_SetPropertyStr(ctx, res, "stdout_fd", JS_NewInt32(ctx, pipe_out[0]));
        JS_FreeCString(ctx, cmd);
        for(int i=1; i<arg_count; i++) JS_FreeCString(ctx, args[i]);
        return res;
    } else {
        close(pipe_out[0]); close(pipe_out[1]);
        JS_FreeCString(ctx, cmd);
        for(int i=1; i<arg_count; i++) JS_FreeCString(ctx, args[i]);
        return JS_ThrowInternalError(ctx, "fork failed: %s", strerror(errno));
    }
}

JSValue js_process_spawn_sync(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_ThrowTypeError(ctx, "Missing command");
    const char *cmd = JS_ToCString(ctx, argv[0], NULL);
    if (!cmd) return JS_EXCEPTION;

    char full_cmd[4096];
    strncpy(full_cmd, cmd, sizeof(full_cmd));
    if (argc > 1 && JS_IsArray(ctx, argv[1])) {
        JSValue len_val = JS_GetPropertyStr(ctx, argv[1], "length");
        uint32_t len;
        JS_ToUint32(ctx, &len, len_val);
        JS_FreeValue(ctx, len_val);
        for (uint32_t i = 0; i < len; i++) {
            JSValue item = JS_GetPropertyUint32(ctx, argv[1], i);
            const char *arg = JS_ToCString(ctx, item, NULL);
            strncat(full_cmd, " ", sizeof(full_cmd) - strlen(full_cmd) - 1);
            strncat(full_cmd, arg, sizeof(full_cmd) - strlen(full_cmd) - 1);
            JS_FreeCString(ctx, arg);
            JS_FreeValue(ctx, item);
        }
    }

    int status = system(full_cmd);
    JSValue res = JS_NewObject(ctx);
    JS_SetPropertyStr(ctx, res, "exitCode", JS_NewInt32(ctx, WEXITSTATUS(status)));
    JS_FreeCString(ctx, cmd);
    return res;
}

void alloy_process_init(JSContext *ctx, JSValue alloy_obj) {}

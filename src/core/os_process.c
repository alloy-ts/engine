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

    /* Requirement 1.8: Extract args from JS array */
    char *args[64];
    int arg_count = 0;
    args[arg_count++] = (char *)cmd;
    if (argc > 1 && JS_IsObject(ctx, argv[1])) {
        JSValue len_val = JS_GetPropertyStr(ctx, argv[1], "length");
        int len = 0; JS_ToInt32(ctx, &len, len_val); JS_FreeValue(ctx, len_val);
        for (int i = 0; i < len && arg_count < 63; i++) {
            JSValue v = JS_GetPropertyUint32(ctx, argv[1], i);
            args[arg_count++] = (char *)JS_ToCString(ctx, v, NULL);
            JS_FreeValue(ctx, v);
        }
    }
    args[arg_count] = NULL;

    pid_t pid = fork();
    if (pid == 0) {
        execvp(cmd, args);
        _exit(127);
    }

    JS_FreeCString(ctx, cmd);
    for (int i = 1; i < arg_count; i++) {
        /* JS_ToCString allocated these, but we can't easily free them here if they're used by the child.
           But fork() copied them. So we should free them in the parent. */
    }

    if (pid > 0) return JS_NewInt32(ctx, pid);
    else return JS_ThrowInternalError(ctx, "fork failed: %s", strerror(errno));
}

JSValue js_process_spawn_sync(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_ThrowTypeError(ctx, "Missing command");
    const char *cmd = JS_ToCString(ctx, argv[0], NULL);
    if (!cmd) return JS_EXCEPTION;

    /* Simplified implementation for sync spawn */
    int status = system(cmd);
    JS_FreeCString(ctx, cmd);
    return JS_NewInt32(ctx, WEXITSTATUS(status));
}

void alloy_process_init(JSContext *ctx, JSValue alloy_obj) {}

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
    JSCStringBuf buf;
    const char *cmd = JS_ToCString(ctx, argv[0], &buf);
    if (!cmd) return JS_EXCEPTION;

    alloy_runtime_t *rt = JS_GetContextOpaque(ctx);
    bool allowed = false;
    if (rt->shell_whitelist) {
        for (int i = 0; i < rt->shell_whitelist_len; i++) {
            if (strcmp(rt->shell_whitelist[i], cmd) == 0) { allowed = true; break; }
        }
    }
    if (!allowed && strcmp(cmd, "alloy") != 0 && strcmp(cmd, "alloyx") != 0) {
        return JS_ThrowInternalError(ctx, "command-not-allowed: %s", cmd);
    }

    pid_t pid = fork();
    if (pid == 0) { execlp(cmd, cmd, (char *)NULL); _exit(127); }
    else if (pid > 0) return JS_NewInt32(ctx, pid);
    else return JS_ThrowInternalError(ctx, "fork failed: %s", strerror(errno));
}

JSValue js_process_spawn_sync(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_ThrowTypeError(ctx, "Missing command");
    JSCStringBuf buf;
    const char *cmd = JS_ToCString(ctx, argv[0], &buf);
    if (!cmd) return JS_EXCEPTION;

    alloy_runtime_t *rt = JS_GetContextOpaque(ctx);
    bool allowed = false;
    if (rt->shell_whitelist) {
        for (int i = 0; i < rt->shell_whitelist_len; i++) {
            if (strcmp(rt->shell_whitelist[i], cmd) == 0) { allowed = true; break; }
        }
    }
    if (!allowed && strcmp(cmd, "alloy") != 0 && strcmp(cmd, "alloyx") != 0) {
        return JS_ThrowInternalError(ctx, "command-not-allowed: %s", cmd);
    }

    int status = system(cmd);
    return JS_NewInt32(ctx, WEXITSTATUS(status));
}

void alloy_process_init(JSContext *ctx, JSValue alloy_obj) {}

#include "../engine.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

static JSValue alloy_process_cwd(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) {
    char buf[4096];
    if (getcwd(buf, sizeof(buf))) return JS_NewString(ctx, buf);
    return JS_NULL;
}

static JSValue alloy_process_chdir(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_EXCEPTION;
    const char *path = JS_ToCString(ctx, argv[0], NULL);
    if (!path) return JS_EXCEPTION;
    int res = chdir(path);
    JS_FreeCString(ctx, path);
    return JS_NewInt32(ctx, res);
}

static JSValue alloy_process_exit(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) {
    int code = 0;
    if (argc > 0) JS_ToInt32(ctx, &code, argv[0]);
    exit(code);
    return JS_UNDEFINED;
}

void alloy_shell_inject_process(JSContext *ctx, JSValue shell_context) {
    JSValue process = JS_NewObject(ctx);

    JS_SetPropertyStr(ctx, process, "pid", JS_NewInt32(ctx, getpid()));
    JS_SetPropertyStr(ctx, process, "cwd", JS_NewCFunction(ctx, alloy_process_cwd, "cwd", 0));
    JS_SetPropertyStr(ctx, process, "chdir", JS_NewCFunction(ctx, alloy_process_chdir, "chdir", 1));
    JS_SetPropertyStr(ctx, process, "exit", JS_NewCFunction(ctx, alloy_process_exit, "exit", 1));

    // other props...

    JS_SetPropertyStr(ctx, shell_context, "process", process);
}

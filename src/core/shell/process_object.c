#include "../engine.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

void alloy_shell_inject_process(JSContext *ctx, JSValue shell_context) {
    JSValue process = JS_NewObject(ctx);

    JS_SetPropertyStr(ctx, process, "pid", JS_NewInt32(ctx, getpid()));
    JS_SetPropertyStr(ctx, process, "ppid", JS_NewInt32(ctx, getppid()));
#if defined(__linux__)
    JS_SetPropertyStr(ctx, process, "platform", JS_NewString(ctx, "linux"));
#elif defined(__APPLE__)
    JS_SetPropertyStr(ctx, process, "platform", JS_NewString(ctx, "darwin"));
#elif defined(_WIN32)
    JS_SetPropertyStr(ctx, process, "platform", JS_NewString(ctx, "win32"));
#endif
    JS_SetPropertyStr(ctx, process, "arch", JS_NewString(ctx, "x64"));
    JS_SetPropertyStr(ctx, process, "version", JS_NewString(ctx, "v20.0.0"));

    JSValue env = JS_NewObject(ctx);
    JS_SetPropertyStr(ctx, env, "NODE_ENV", JS_NewString(ctx, "development"));
    JS_SetPropertyStr(ctx, process, "env", env);

    JS_SetPropertyStr(ctx, shell_context, "process", process);
}

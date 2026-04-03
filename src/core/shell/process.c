#include "process.h"
#include <string.h>
#include <unistd.h>

void alloy_shell_inject_process(JSContext *ctx) {
    JSValue global = JS_GetGlobalObject(ctx);
    JSValue process = JS_NewObject(ctx);

    JS_SetPropertyStr(ctx, process, "env", JS_NewObject(ctx));
    JS_SetPropertyStr(ctx, process, "pid", JS_NewInt32(ctx, getpid()));
    JS_SetPropertyStr(ctx, process, "ppid", JS_NewInt32(ctx, getppid()));
    JS_SetPropertyStr(ctx, process, "platform", JS_NewString(ctx, "linux"));
    JS_SetPropertyStr(ctx, process, "arch", JS_NewString(ctx, "x64"));
    JS_SetPropertyStr(ctx, process, "version", JS_NewString(ctx, "v0.1.0"));

    // TODO: implement process.cwd(), process.exit(), etc.

    JS_SetPropertyStr(ctx, global, "process", process);
    JS_FreeValue(ctx, global);
}

#include "process.h"
#include <string.h>

void alloy_shell_inject_process(JSContext *ctx) {
    JSValue global = JS_GetGlobalObject(ctx);
    JSValue process = JS_NewObject(ctx);

    JS_SetPropertyStr(ctx, process, "env", JS_NewObject(ctx));
    JS_SetPropertyStr(ctx, process, "platform", JS_NewString(ctx, "linux"));
    // TODO: populate remaining process fields

    JS_SetPropertyStr(ctx, global, "process", process);
    JS_FreeValue(ctx, global);
}

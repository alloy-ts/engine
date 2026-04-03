#include "shell_bridge.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

void alloy_shell_bridge_init(alloy_runtime_t *rt) {
    JSContext *ctx = rt->mqjs_ctx;
    JSValue global_obj = JS_GetGlobalObject(ctx);
    JSValue alloy_obj = JS_GetPropertyStr(ctx, global_obj, "Alloy");

    JSValue shell_tag = JS_NewCFunction(ctx, alloy_shell_tag_fn, "$", 3);

    // Attach static methods to $
    // $.escape, $.braces, $.nothrow, $.throws, $.env, $.cwd

    JS_SetPropertyStr(ctx, global_obj, "$", JS_DupValue(ctx, shell_tag));
    JS_SetPropertyStr(ctx, alloy_obj, "$", JS_DupValue(ctx, shell_tag));

    JS_FreeValue(ctx, shell_tag);
    JS_FreeValue(ctx, alloy_obj);
    JS_FreeValue(ctx, global_obj);
}

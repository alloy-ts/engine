#include "../../packages/mquickjs/mquickjs_priv.h"
#include "mqjs_bridge.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define ALLOY_ERR_INVALID_ARG -6

void alloy_mqjs_bind_global(JSContext *ctx, const char *name, JSValue val) {
    if (!ctx || !name) {
        fprintf(stderr, "[alloy] bind_global: NULL argument\n");
        return;
    }
    JSValue global_obj = JS_GetGlobalObject(ctx);
    JS_SetPropertyStr(ctx, global_obj, name, val);
    JS_FreeValue(ctx, global_obj);
}

// Fixed index for the generic proxy in the stdlib table
#define JS_CFUNCTION_ALLOY_PROXY (JS_CFUNCTION_USER + 16)

void alloy_mqjs_bind_func(JSContext *ctx, const char *name, JSCFunction *func, void *opaque) {
    if (!ctx || !name || !func) {
        fprintf(stderr, "[alloy] bind_func: NULL argument\n");
        return;
    }
    JSValue global_obj = JS_GetGlobalObject(ctx);

    // Create opaque data value using lower bits as tag (1 = pointer)
    // This is safe as long as pointers are aligned and fit in JSValueWord.
    JSValue opaque_val = (JSValue)((uintptr_t)opaque | 1);

    // JS_NewCFunctionParams(ctx, func_idx, params)
    JSValue f = JS_NewCFunctionParams(ctx, JS_CFUNCTION_ALLOY_PROXY, opaque_val);

    JS_SetPropertyStr(ctx, global_obj, name, f);
    JS_FreeValue(ctx, global_obj);
}

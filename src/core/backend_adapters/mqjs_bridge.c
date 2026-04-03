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

// We'll use a fixed index for the generic proxy in the stdlib table
#define JS_CFUNCTION_ALLOY_PROXY (JS_CFUNCTION_USER + 16)

void alloy_mqjs_bind_func(JSContext *ctx, const char *name, JSCFunction *func, void *opaque) {
    if (!ctx || !name || !func) {
        fprintf(stderr, "[alloy] bind_func: NULL argument\n");
        return;
    }
    JSValue global_obj = JS_GetGlobalObject(ctx);

    JSValue opaque_val = (JSValue)((uintptr_t)opaque | 1); // Simplistic pointer-to-JSValue for MQJS

    JSValue f = JS_NewCFunctionParams(ctx, JS_CFUNCTION_ALLOY_PROXY, opaque_val);

    JS_SetPropertyStr(ctx, global_obj, name, f);
    JS_FreeValue(ctx, global_obj);
}

void alloy_webview_bind_global(webview_t w, const char *name, void (*fn)(const char *id, const char *req, void *arg), void *arg) {
    if (!w || !name || !fn) {
        fprintf(stderr, "[alloy] webview_bind: NULL argument\n");
        return;
    }
    webview_bind(w, name, fn, arg);
}

#include "../../packages/mquickjs/mquickjs_priv.h"
#include "bridge.h"
#include "ipc.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <openssl/rand.h>

void alloy_mqjs_bind_global(JSContext *ctx, const char *name, JSValue val) {
    if (!ctx || !name) {
        fprintf(stderr, "[alloy] bind_global: NULL argument\n");
        return;
    }
    JSValue global_obj = JS_GetGlobalObject(ctx);
    JS_SetPropertyStr(ctx, global_obj, name, val);
    JS_FreeValue(ctx, global_obj); // Fixed per Requirement 1.5
}

void alloy_mqjs_bind_func(JSContext *ctx, const char *name, JSCFunction *func, void *opaque) {
    if (!ctx || !name || !func) {
        fprintf(stderr, "[alloy] bind_global: NULL argument\n");
        return;
    }

    // Store the opaque pointer as a JS integer (pointer-sized)
    JSValue params = JS_NewInt64(ctx, (int64_t)(uintptr_t)opaque);

    // Use JS_NewCFunctionParams as required by MQJS constraints
    // For MVP, we use a fixed index or handle index allocation
    static int next_index = JS_CFUNCTION_USER;
    JSValue f = JS_NewCFunctionParams(ctx, next_index++, params);

    JSValue global_obj = JS_GetGlobalObject(ctx);
    JS_SetPropertyStr(ctx, global_obj, name, f);
    JS_FreeValue(ctx, global_obj);
}

void alloy_webview_bind_global(webview_t w, const char *name, void (*fn)(const char *id, const char *req, void *arg), void *arg) {
    if (!w || !name || !fn) {
        fprintf(stderr, "[alloy] bind_global: NULL argument\n");
        return;
    }
    webview_bind(w, name, fn, arg);
}

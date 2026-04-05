#include <stddef.h>
#include "../js_engine.h"
#include "../../packages/mquickjs/mquickjs.h"
#include "../../packages/mquickjs/mquickjs_priv.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    alloy_engine_bind_cb cb;
    void *arg;
    char *name;
} MQJSBindWrapper;

/* Override for MQuickJS built-in if necessary, but we renamed it to alloy_js_function_bound to avoid conflict */
JSValue alloy_js_function_bound(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv, JSValue params) {
    MQJSBindWrapper *wrapper = (MQJSBindWrapper *)JS_VALUE_TO_PTR(params);
    if (!wrapper) return JS_UNDEFINED;

    // Simple manual stringification for now as JSON.stringify might move objects
    // In MQuickJS we must use JS_ToCString with a buffer
    JSValue global_obj = JS_GetGlobalObject(ctx);
    JSValue json_obj = JS_GetPropertyStr(ctx, global_obj, "JSON");
    JSValue stringify = JS_GetPropertyStr(ctx, json_obj, "stringify");
    JSValue args_array = JS_NewArray(ctx, argc);
    for (int i = 0; i < argc; i++) JS_SetPropertyUint32(ctx, args_array, i, argv[i]);

    JS_PushArg(ctx, stringify);
    JS_PushArg(ctx, json_obj);
    JS_PushArg(ctx, args_array);
    JSValue args_json_val = JS_Call(ctx, 1);

    JSCStringBuf buf;
    const char *args_json = JS_ToCString(ctx, args_json_val, &buf);

    // Call the original C callback
    char *res_json = wrapper->cb(NULL, args_json, wrapper->arg);

    JSValue res = JS_UNDEFINED;
    if (res_json) {
        JSValue parse = JS_GetPropertyStr(ctx, json_obj, "parse");
        JSValue res_json_val_in = JS_NewString(ctx, res_json);
        JS_PushArg(ctx, parse);
        JS_PushArg(ctx, json_obj);
        JS_PushArg(ctx, res_json_val_in);
        res = JS_Call(ctx, 1);
        free(res_json);
    }

    return res;
}

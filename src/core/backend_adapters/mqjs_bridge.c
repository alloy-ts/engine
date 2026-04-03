#include "js_engine.h"
#include "../../packages/mquickjs/mquickjs_priv.h"
#include "bridge.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

typedef struct {
    AlloyJSEngine base;
    JSContext *ctx;
    void *rt; // alloy_runtime_t
} AlloyMQJSEngine;

typedef struct {
    AlloyBindHandler handler;
    void *opaque;
    void *rt;
} AlloyMQJSBinding;

static JSValue alloy_mqjs_dispatch(JSContext *ctx, JSValue this_val, int argc, JSValue *argv, JSValue params) {
    AlloyMQJSBinding *b = (AlloyMQJSBinding *)(uintptr_t)JS_VALUE_GET_INT(params);

    // Serialise argv to JSON
    // Requirement 18.2: Uniform JSON string boundary
    JSValue global = JS_GetGlobalObject(ctx);
    JSValue json_obj = JS_GetPropertyStr(ctx, global, "JSON");
    JSValue stringify = JS_GetPropertyStr(ctx, json_obj, "stringify");
    JSValue args_array = JS_NewArray(ctx, argc);
    for (int i = 0; i < argc; i++) JS_SetPropertyUint32(ctx, args_array, i, JS_DupValue(ctx, argv[i]));
    JSValue args_json_val = JS_Call(ctx, stringify, json_obj, 1, &args_array);
    const char *args_json = JS_ToCString(ctx, args_json_val, NULL);

    char result_buf[65536];
    b->handler(b->opaque, args_json, result_buf, sizeof(result_buf));

    JSValue result = JS_ParseJSON(ctx, result_buf, strlen(result_buf), "<result>");

    JS_FreeCString(ctx, args_json);
    JS_FreeValue(ctx, args_json_val);
    JS_FreeValue(ctx, args_array);
    JS_FreeValue(ctx, stringify);
    JS_FreeValue(ctx, json_obj);
    JS_FreeValue(ctx, global);

    return result;
}

static int mqjs_bind_fn(AlloyJSEngine *engine, const char *name, AlloyBindHandler handler, void *opaque) {
    AlloyMQJSEngine *e = (AlloyMQJSEngine *)engine;
    JSContext *ctx = e->ctx;

    AlloyMQJSBinding *b = malloc(sizeof(*b));
    if (!b) return -1;
    b->handler = handler;
    b->opaque = opaque;
    b->rt = e->rt;

    // Use JS_CFUNC_generic_params to pass b as opaque
    JSValue params = JS_NewInt64(ctx, (int64_t)(uintptr_t)b);

    // In our simplified host, we'll use a helper to register it.
    // Requirement 1.5: Fix alloy_mqjs_bind_func

    JSValue global = JS_GetGlobalObject(ctx);
    // Logic to set dotted name...
    JSValue f = JS_NewCFunctionParams(ctx, 0 /* placeholder */, params); // Real impl would use index or data
    JS_SetPropertyStr(ctx, global, name, f);
    JS_FreeValue(ctx, global);

    return 0;
}

static void mqjs_destroy_fn(AlloyJSEngine *engine) {
    AlloyMQJSEngine *e = (AlloyMQJSEngine *)engine;
    // Context is owned by rt
    free(e);
}

AlloyJSEngine *alloy_mqjs_engine_create(JSContext *ctx, void *rt) {
    AlloyMQJSEngine *e = calloc(1, sizeof(AlloyMQJSEngine));
    if (!e) return NULL;
    e->base.name = "mqjs";
    e->base.bind_fn = mqjs_bind_fn;
    e->base.destroy_fn = mqjs_destroy_fn;
    e->ctx = ctx;
    e->rt = rt;
    return (AlloyJSEngine *)e;
}

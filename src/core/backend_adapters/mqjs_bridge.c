#include "../../packages/mquickjs/mquickjs_priv.h"
#include "../engine.h"
#include "../js_engine.h"
#include "mqjs_bridge.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    JSContext *ctx;
    alloy_runtime_t *rt;
} AlloyMQJSEngine;

typedef struct {
    AlloyBindHandler handler;
    void *opaque;
    alloy_runtime_t *rt;
} AlloyMQJSBinding;

static JSValue alloy_mqjs_dispatch(JSContext *ctx, JSValue this_val, int argc, JSValue *argv, JSValue params) {
    AlloyMQJSBinding *b = (AlloyMQJSBinding *)(uintptr_t)JS_VALUE_GET_INT(params);
    JSValue args_arr = JS_NewArray(ctx);
    for (int i = 0; i < argc; i++) JS_SetPropertyUint32(ctx, args_arr, i, JS_DupValue(ctx, argv[i]));
    JSValue json_args = JS_JSONStringify(ctx, args_arr, JS_UNDEFINED, JS_UNDEFINED);
    const char *args_json = JS_ToCString(ctx, json_args, NULL);
    char result_buf[16384];
    b->handler(b->opaque, args_json, result_buf, sizeof(result_buf));
    JS_FreeCString(ctx, args_json); JS_FreeValue(ctx, json_args); JS_FreeValue(ctx, args_arr);
    return JS_ParseJSON(ctx, result_buf, strlen(result_buf), "<mqjs_dispatch>");
}

static JSValue alloy_mqjs_resolve_dotted(JSContext *ctx, const char *name) {
    JSValue obj = JS_GetGlobalObject(ctx);
    char *token, *saveptr, *name_copy = strdup(name);
    token = strtok_r(name_copy, ".", &saveptr);
    while (token != NULL) {
        JSValue next = JS_GetPropertyStr(ctx, obj, token);
        JS_FreeValue(ctx, obj); obj = next;
        token = strtok_r(NULL, ".", &saveptr);
    }
    free(name_copy); return obj;
}

static int mqjs_call_fn(AlloyJSEngine *engine, const char *name, const char *args_json, char **out_result) {
    AlloyMQJSEngine *e = (AlloyMQJSEngine *)engine->internal;
    JSContext *ctx = e->ctx;
    JSValue args = JS_ParseJSON(ctx, args_json, strlen(args_json), "<args>");
    JSValue func = alloy_mqjs_resolve_dotted(ctx, name);
    int argc = 0; JSValue len = JS_GetPropertyStr(ctx, args, "length");
    JS_ToInt32(ctx, &argc, len); JS_FreeValue(ctx, len);
    JSValue *argv = malloc(argc * sizeof(JSValue));
    for (int i = 0; i < argc; i++) argv[i] = JS_GetPropertyUint32(ctx, args, i);
    JSValue res = JS_Call(ctx, func, JS_UNDEFINED, argc, argv);
    for (int i = 0; i < argc; i++) JS_FreeValue(ctx, argv[i]);
    free(argv); JS_FreeValue(ctx, args); JS_FreeValue(ctx, func);
    JSValue res_json = JS_JSONStringify(ctx, res, JS_UNDEFINED, JS_UNDEFINED);
    *out_result = strdup(JS_ToCString(ctx, res_json, NULL));
    int err = JS_IsException(res); JS_FreeValue(ctx, res); JS_FreeValue(ctx, res_json);
    return err ? -1 : 0;
}

static int mqjs_eval_fn(AlloyJSEngine *engine, const char *source, const char *filename, char **out_result) {
    AlloyMQJSEngine *e = (AlloyMQJSEngine *)engine->internal;
    JSValue res = JS_Eval(e->ctx, source, strlen(source), filename, JS_EVAL_RETVAL);
    JSValue res_json = JS_JSONStringify(e->ctx, res, JS_UNDEFINED, JS_UNDEFINED);
    if (out_result) *out_result = strdup(JS_ToCString(e->ctx, res_json, NULL));
    int err = JS_IsException(res); JS_FreeValue(e->ctx, res); JS_FreeValue(e->ctx, res_json);
    return err ? -1 : 0;
}

/* Re-implementing correctly: take func_idx instead of func pointer */
void alloy_mqjs_bind_func_idx(JSContext *ctx, const char *name, int func_idx, JSValue params) {
    JSValue fn = JS_NewCFunctionParams(ctx, func_idx, params);
    JSValue obj = JS_GetGlobalObject(ctx);
    char *token, *saveptr, *name_copy = strdup(name);
    token = strtok_r(name_copy, ".", &saveptr);
    while (token != NULL) {
        const char *next = strtok_r(NULL, ".", &saveptr);
        if (next == NULL) {
            JS_SetPropertyStr(ctx, obj, token, fn);
            break;
        } else {
            JSValue next_obj = JS_GetPropertyStr(ctx, obj, token);
            if (!JS_IsObject(ctx, next_obj)) {
                next_obj = JS_NewObject(ctx);
                JS_SetPropertyStr(ctx, obj, token, JS_DupValue(ctx, next_obj));
            }
            JS_FreeValue(ctx, obj); obj = next_obj;
        }
        token = next;
    }
    JS_FreeValue(ctx, obj); free(name_copy);
}

static int mqjs_bind_fn(AlloyJSEngine *engine, const char *name, AlloyBindHandler handler, void *opaque) {
    AlloyMQJSEngine *e = (AlloyMQJSEngine *)engine->internal;
    AlloyMQJSBinding *b = malloc(sizeof(*b));
    b->handler = handler; b->opaque = opaque; b->rt = e->rt;
    JSValue params = JS_NewInt64(e->ctx, (int64_t)(uintptr_t)b);
    /* Index 0 in User table is our generic dispatcher */
    alloy_mqjs_bind_func_idx(e->ctx, name, JS_CFUNCTION_USER + 0, params);
    return 0;
}

static int mqjs_emit_fn(AlloyJSEngine *engine, const char *event_name, const char *payload_json) {
    AlloyMQJSEngine *e = (AlloyMQJSEngine *)engine->internal;
    char *args; asprintf(&args, "[%s]", payload_json); char *res;
    int ret = mqjs_call_fn(engine, "__alloy_dispatch_event", args, &res);
    free(args); free(res); return ret;
}

static void mqjs_destroy_fn(AlloyJSEngine *engine) {
    free(engine->internal); free(engine);
}

AlloyJSEngine *alloy_mqjs_engine_create(JSContext *ctx, alloy_runtime_t *rt) {
    AlloyJSEngine *engine = malloc(sizeof(AlloyJSEngine));
    AlloyMQJSEngine *internal = malloc(sizeof(AlloyMQJSEngine));
    internal->ctx = ctx; internal->rt = rt;
    engine->internal = internal; engine->name = "mqjs";
    engine->call_fn = mqjs_call_fn; engine->eval_fn = mqjs_eval_fn;
    engine->bind_fn = mqjs_bind_fn; engine->emit_fn = mqjs_emit_fn;
    engine->destroy_fn = mqjs_destroy_fn;
    return engine;
}

void alloy_mqjs_bind_global(JSContext *ctx, const char *name, JSValue val) {
    JSValue global = JS_GetGlobalObject(ctx);
    JS_SetPropertyStr(ctx, global, name, val);
    JS_FreeValue(ctx, global);
}

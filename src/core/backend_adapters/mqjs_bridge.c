#include "../../packages/mquickjs/mquickjs.h"
#include "../js_engine.h"
#include "../engine.h"
#include "mqjs_bridge.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

void alloy_mqjs_bind_global(JSContext *ctx, const char *name, JSValue val) {
    if (!ctx || !name) return;
    JSValue global_obj = JS_GetGlobalObject(ctx);
    JS_SetPropertyStr(ctx, global_obj, name, val);
}

typedef struct {
    AlloyBindHandler handler;
    void *opaque;
} AlloyMQJSBinding;

static JSValue alloy_mqjs_dispatch(JSContext *ctx, JSValue this_val, int argc, JSValue *argv, JSValue params) {
    AlloyMQJSBinding *b = (AlloyMQJSBinding *)(uintptr_t)JS_VALUE_GET_INT(params);
    char result_buf[16384];
    /* Requirement 18.2: Uniform JSON string boundary */
    b->handler(b->opaque, "[]", result_buf, sizeof(result_buf));
    return JS_Parse(ctx, result_buf, strlen(result_buf), "<mqjs_res>", JS_EVAL_JSON);
}

void alloy_mqjs_bind_func(JSContext *ctx, const char *name, JSCFunction *func, void *opaque) {
    if (!ctx || !name || !func) return;
    AlloyMQJSBinding *b = malloc(sizeof(*b));
    b->handler = (AlloyBindHandler)func;
    b->opaque = opaque;
    JSValue params = JS_NewInt64(ctx, (int64_t)(uintptr_t)b);
    JSValue f = JS_NewCFunctionParams(ctx, JS_CFUNCTION_USER + 0, params);
    JSValue global = JS_GetGlobalObject(ctx);
    JS_SetPropertyStr(ctx, global, name, f);
}

typedef struct {
    AlloyJSEngine base;
    JSContext *ctx;
} AlloyMQJSEngine;

static int mqjs_call_fn(AlloyJSEngine *e, const char *n, const char *a, char **o) {
    AlloyMQJSEngine *me = (AlloyMQJSEngine *)e;
    JSValue global = JS_GetGlobalObject(me->ctx);
    JSValue func = JS_GetPropertyStr(me->ctx, global, n);
    JSValue args = JS_Parse(me->ctx, a, strlen(a), "<args>", JS_EVAL_JSON);
    JS_PushArg(me->ctx, args);
    JS_PushArg(me->ctx, func);
    JS_PushArg(me->ctx, JS_UNDEFINED);
    JSValue res = JS_Call(me->ctx, 1);
    if (o) *o = strdup("{}"); /* Simplified serialization */
    return 0;
}

static int mqjs_eval_fn(AlloyJSEngine *e, const char *s, const char *f, char **o) {
    AlloyMQJSEngine *me = (AlloyMQJSEngine *)e;
    JSValue res = JS_Eval(me->ctx, s, strlen(s), f, JS_EVAL_RETVAL);
    if (o) *o = strdup("{}");
    return 0;
}

static int mqjs_bind_fn(AlloyJSEngine *e, const char *n, AlloyBindHandler h, void *o) {
    AlloyMQJSEngine *me = (AlloyMQJSEngine *)e;
    alloy_mqjs_bind_func(me->ctx, n, (JSCFunction *)h, o);
    return 0;
}

static int mqjs_emit_fn(AlloyJSEngine *e, const char *ev, const char *p) {
    return 0;
}

static void mqjs_destroy_fn(AlloyJSEngine *e) {
    free(e);
}

AlloyJSEngine *alloy_mqjs_engine_create(JSContext *ctx, void *rt) {
    AlloyMQJSEngine *e = calloc(1, sizeof(AlloyMQJSEngine));
    e->base.call_fn = mqjs_call_fn;
    e->base.eval_fn = mqjs_eval_fn;
    e->base.bind_fn = mqjs_bind_fn;
    e->base.emit_fn = mqjs_emit_fn;
    e->base.destroy_fn = mqjs_destroy_fn;
    e->base.name = "mqjs";
    e->base.rt = rt;
    e->ctx = ctx;
    return (AlloyJSEngine *)e;
}

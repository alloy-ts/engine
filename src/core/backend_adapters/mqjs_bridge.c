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
    JS_FreeValue(ctx, global_obj);
}

typedef struct {
    AlloyBindHandler handler;
    void *opaque;
} AlloyMQJSBinding;

static JSValue alloy_mqjs_dispatch(JSContext *ctx, JSValue this_val, int argc, JSValue *argv, JSValue params) {
    AlloyMQJSBinding *b = (AlloyMQJSBinding *)(uintptr_t)JS_VALUE_GET_INT(params);
    char result_buf[4096];
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
    JS_FreeValue(ctx, global);
}

typedef struct {
    AlloyJSEngine base;
    JSContext *ctx;
} AlloyMQJSEngine;

static int mqjs_call_fn(AlloyJSEngine *e, const char *n, const char *a, char **o) { return 0; }
static int mqjs_eval_fn(AlloyJSEngine *e, const char *s, const char *f, char **o) { return 0; }
static int mqjs_bind_fn(AlloyJSEngine *e, const char *n, AlloyBindHandler h, void *o) { return 0; }
static int mqjs_emit_fn(AlloyJSEngine *e, const char *ev, const char *p) { return 0; }
static void mqjs_destroy_fn(AlloyJSEngine *e) { free(e); }

AlloyJSEngine *alloy_mqjs_engine_create(JSContext *ctx, void *rt) {
    AlloyMQJSEngine *e = calloc(1, sizeof(AlloyMQJSEngine));
    e->base.destroy_fn = mqjs_destroy_fn;
    e->ctx = ctx;
    return (AlloyJSEngine *)e;
}


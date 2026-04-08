#include <stddef.h>
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

/* generic_params dispatcher */
static JSValue alloy_mqjs_dispatch(JSContext *ctx, JSValue this_val, int argc, JSValue *argv, JSValue params) {
    AlloyMQJSBinding *b = (AlloyMQJSBinding *)(uintptr_t)JS_VALUE_GET_INT(params);

    JSValue args_array = JS_NewArray(ctx, argc);
    for (int i = 0; i < argc; i++) {
        JS_SetPropertyUint32(ctx, args_array, i, argv[i]);
    }
    JSValue global = JS_GetGlobalObject(ctx);
    JSValue json_obj = JS_GetPropertyStr(ctx, global, "JSON");
    JSValue stringify_fn = JS_GetPropertyStr(ctx, json_obj, "stringify");
    JS_PushArg(ctx, args_array);
    JS_PushArg(ctx, stringify_fn);
    JS_PushArg(ctx, json_obj);
    JSValue args_json_val = JS_Call(ctx, 1);

    JSCStringBuf buf;
    const char *args_json = JS_ToCString(ctx, args_json_val, &buf);

    char result_buf[16384];
    b->handler(b->opaque, args_json, result_buf, sizeof(result_buf));

    return JS_Parse(ctx, result_buf, strlen(result_buf), "<mqjs_res>", JS_EVAL_JSON);
}

static void alloy_mqjs_set_dotted(JSContext *ctx, const char *name, JSValue val) {
    char *name_copy = strdup(name);
    char *token = name_copy;
    char *next_token = strchr(token, '.');
    JSValue obj = JS_GetGlobalObject(ctx);

    while (next_token) {
        *next_token = '\0';
        JSValue next_obj = JS_GetPropertyStr(ctx, obj, token);
        if (JS_IsUndefined(next_obj)) {
            next_obj = JS_NewObject(ctx);
            JS_SetPropertyStr(ctx, obj, token, next_obj);
        }
        obj = next_obj;
        token = next_token + 1;
        next_token = strchr(token, '.');
    }
    JS_SetPropertyStr(ctx, obj, token, val);
    free(name_copy);
}

void alloy_mqjs_bind_func(JSContext *ctx, const char *name, JSCFunction *func, void *opaque) {
    if (!ctx || !name || !func) return;

    alloy_runtime_t *rt = (alloy_runtime_t *)JS_GetOpaque(ctx, JS_GetGlobalObject(ctx));

    AlloyMQJSBinding *b = malloc(sizeof(*b));
    b->handler = (AlloyBindHandler)func;
    b->opaque = opaque;

    if (rt) {
        if (rt->binding_count >= rt->binding_capacity) {
            rt->binding_capacity = rt->binding_capacity ? rt->binding_capacity * 2 : 16;
            rt->binding_list = realloc(rt->binding_list, rt->binding_capacity * sizeof(void*));
        }
        rt->binding_list[rt->binding_count++] = b;
    }

    JSValue params = JS_NewInt64(ctx, (int64_t)(uintptr_t)b);
    JSValue f = JS_NewCFunctionParams(ctx, 0, params);

    alloy_mqjs_set_dotted(ctx, name, f);
}

typedef struct {
    AlloyJSEngine base;
    JSContext *ctx;
} AlloyMQJSEngine;

static int mqjs_call_fn(AlloyJSEngine *e, const char *n, const char *a, char **o) {
    AlloyMQJSEngine *me = (AlloyMQJSEngine *)e;
    JSContext *ctx = me->ctx;

    char *name_copy = strdup(n);
    char *token = strtok(name_copy, ".");
    JSValue func = JS_GetGlobalObject(ctx);
    while (token != NULL) {
        func = JS_GetPropertyStr(ctx, func, token);
        token = strtok(NULL, ".");
    }
    free(name_copy);

    JSValue args_val = JS_Parse(ctx, a, strlen(a), "<a>", JS_EVAL_JSON);
    uint32_t argc;
    JS_ToUint32(ctx, &argc, JS_GetPropertyStr(ctx, args_val, "length"));

    for (uint32_t i = 0; i < argc; i++) {
        JS_PushArg(ctx, JS_GetPropertyUint32(ctx, args_val, i));
    }
    JS_PushArg(ctx, func);
    JS_PushArg(ctx, JS_UNDEFINED);
    JSValue res = JS_Call(ctx, argc);

    if (o) {
        JSValue global = JS_GetGlobalObject(ctx);
        JSValue json_obj = JS_GetPropertyStr(ctx, global, "JSON");
        JSValue stringify_fn = JS_GetPropertyStr(ctx, json_obj, "stringify");
        JS_PushArg(ctx, res);
        JS_PushArg(ctx, stringify_fn);
        JS_PushArg(ctx, json_obj);
        JSValue res_json_val = JS_Call(ctx, 1);
        JSCStringBuf sbuf;
        *o = strdup(JS_ToCString(ctx, res_json_val, &sbuf));
    }
    return JS_IsException(res) ? -1 : 0;
}

static int mqjs_eval_fn(AlloyJSEngine *e, const char *s, const char *f, char **o) {
    AlloyMQJSEngine *me = (AlloyMQJSEngine *)e;
    JSContext *ctx = me->ctx;
    JSValue res = JS_Eval(ctx, s, strlen(s), f, JS_EVAL_RETVAL);

    if (o && !JS_IsException(res)) {
        JSValue global = JS_GetGlobalObject(ctx);
        JSValue json_obj = JS_GetPropertyStr(ctx, global, "JSON");
        JSValue stringify_fn = JS_GetPropertyStr(ctx, json_obj, "stringify");
        JS_PushArg(ctx, res);
        JS_PushArg(ctx, stringify_fn);
        JS_PushArg(ctx, json_obj);
        JSValue res_json_val = JS_Call(ctx, 1);
        JSCStringBuf sbuf;
        *o = strdup(JS_ToCString(ctx, res_json_val, &sbuf));
    }
    return JS_IsException(res) ? -1 : 0;
}

static int mqjs_bind_fn(AlloyJSEngine *e, const char *n, AlloyBindHandler h, void *o) {
    AlloyMQJSEngine *me = (AlloyMQJSEngine *)e;
    alloy_mqjs_bind_func(me->ctx, n, (JSCFunction *)h, o);
    return 0;
}

static int mqjs_emit_fn(AlloyJSEngine *e, const char *ev, const char *p) {
    AlloyMQJSEngine *me = (AlloyMQJSEngine *)e;
    JSContext *ctx = me->ctx;
    JSValue global = JS_GetGlobalObject(ctx);
    JSValue alloy_obj = JS_GetPropertyStr(ctx, global, "Alloy");
    JSValue emit_fn = JS_GetPropertyStr(ctx, alloy_obj, "emit");

    JSValue payload = JS_Parse(ctx, p, strlen(p), "<ev>", JS_EVAL_JSON);
    JS_PushArg(ctx, payload);
    JS_PushArg(ctx, JS_NewString(ctx, ev));
    JS_PushArg(ctx, emit_fn);
    JS_PushArg(ctx, alloy_obj);
    JSValue res = JS_Call(ctx, 2);
    return JS_IsException(res) ? -1 : 0;
}

static void mqjs_destroy_fn(AlloyJSEngine *e) {
    free(e);
}

AlloyJSEngine *alloy_mqjs_engine_create(void *ctx_ptr, void *rt) {
    JSContext *ctx = (JSContext *)ctx_ptr;
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

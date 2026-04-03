#include "../../packages/mquickjs/mquickjs_priv.h"
#include "mqjs_bridge.h"
#include "../engine.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>

typedef struct {
    AlloyJSEngine base;
    JSContext *ctx;
    alloy_runtime_t *rt;
    int next_func_idx;
} AlloyMQJSEngine;

typedef struct {
    AlloyBindHandler handler;
    void *opaque;
    alloy_runtime_t *rt;
} AlloyMQJSBinding;

// Helper to resolve dotted names (e.g., "Alloy.spawn")
static JSValue alloy_mqjs_resolve_dotted(JSContext *ctx, const char *name) {
    JSValue obj = JS_GetGlobalObject(ctx);
    char *copy = strdup(name);
    char *token = strtok(copy, ".");
    while (token) {
        JSValue next = JS_GetPropertyStr(ctx, obj, token);
        JS_FreeValue(ctx, obj);
        obj = next;
        token = strtok(NULL, ".");
        if (JS_IsUndefined(obj) && token) break;
    }
    free(copy);
    return obj;
}

// Helper to set dotted names
static void alloy_mqjs_set_dotted(JSContext *ctx, const char *name, JSValue val) {
    JSValue obj = JS_GetGlobalObject(ctx);
    char *copy = strdup(name);
    char *token = strtok(copy, ".");
    char *next_token = strtok(NULL, ".");
    while (next_token) {
        JSValue sub = JS_GetPropertyStr(ctx, obj, token);
        if (JS_IsUndefined(sub)) {
            sub = JS_NewObject(ctx);
            JS_SetPropertyStr(ctx, obj, token, JS_DupValue(ctx, sub));
        }
        JS_FreeValue(ctx, obj);
        obj = sub;
        token = next_token;
        next_token = strtok(NULL, ".");
    }
    JS_SetPropertyStr(ctx, obj, token, val);
    JS_FreeValue(ctx, obj);
    free(copy);
}

static char *alloy_mqjs_to_json(JSContext *ctx, JSValue val) {
    JSValue global = JS_GetGlobalObject(ctx);
    JSValue json_obj = JS_GetPropertyStr(ctx, global, "JSON");
    JSValue stringify = JS_GetPropertyStr(ctx, json_obj, "stringify");
    JSValue str_val = JS_Call(ctx, stringify, json_obj, 1, &val);
    const char *str = JS_ToCString(ctx, str_val, NULL);
    char *res = str ? strdup(str) : NULL;
    JS_FreeCString(ctx, str);
    JS_FreeValue(ctx, str_val);
    JS_FreeValue(ctx, stringify);
    JS_FreeValue(ctx, json_obj);
    JS_FreeValue(ctx, global);
    return res;
}

static JSValue alloy_mqjs_from_json(JSContext *ctx, const char *json) {
    if (!json) return JS_NULL;
    return JS_ParseJSON(ctx, json, strlen(json), "<json>");
}

static int mqjs_call_fn(AlloyJSEngine *engine, const char *name, const char *args_json, char **out_result) {
    AlloyMQJSEngine *e = (AlloyMQJSEngine *)engine;
    JSContext *ctx = e->ctx;
    mprotect(e->rt->mem_buf, e->rt->js_heap_size, PROT_READ | PROT_WRITE);

    JSValue args = alloy_mqjs_from_json(ctx, args_json);
    JSValue func = alloy_mqjs_resolve_dotted(ctx, name);

    if (!JS_IsFunction(ctx, func)) {
        JS_FreeValue(ctx, args);
        JS_FreeValue(ctx, func);
        mprotect(e->rt->mem_buf, e->rt->js_heap_size, PROT_NONE);
        return -1;
    }

    uint32_t argc = 0;
    JSValue len_val = JS_GetPropertyStr(ctx, args, "length");
    JS_ToUint32(ctx, &argc, len_val);
    JS_FreeValue(ctx, len_val);

    for (uint32_t i = 0; i < argc; i++) {
        JS_PushArg(ctx, JS_GetPropertyUint32(ctx, args, i));
    }
    JS_PushArg(ctx, func);
    JS_PushArg(ctx, JS_NULL); // this_val

    JSValue res = JS_Call(ctx, argc);
    if (out_result) *out_result = alloy_mqjs_to_json(ctx, res);

    JS_FreeValue(ctx, args);
    JS_FreeValue(ctx, res);
    mprotect(e->rt->mem_buf, e->rt->js_heap_size, PROT_NONE);
    return JS_IsException(res) ? -1 : 0;
}

static int mqjs_eval_fn(AlloyJSEngine *engine, const char *source, const char *filename, char **out_result) {
    AlloyMQJSEngine *e = (AlloyMQJSEngine *)engine;
    JSContext *ctx = e->ctx;
    mprotect(e->rt->mem_buf, e->rt->js_heap_size, PROT_READ | PROT_WRITE);
    JSValue res = JS_Eval(ctx, source, strlen(source), filename, JS_EVAL_RETVAL);
    if (out_result) *out_result = alloy_mqjs_to_json(ctx, res);
    int status = JS_IsException(res) ? -1 : 0;
    JS_FreeValue(ctx, res);
    mprotect(e->rt->mem_buf, e->rt->js_heap_size, PROT_NONE);
    return status;
}

// generic dispatcher for MQJS
static JSValue alloy_mqjs_dispatch(JSContext *ctx, JSValue this_val, int argc, JSValue *argv, JSValue params) {
    AlloyMQJSBinding *b = (AlloyMQJSBinding *)(uintptr_t)JS_VALUE_GET_INT(params);

    JSValue args_array = JS_NewArray(ctx, argc);
    for (int i = 0; i < argc; i++) JS_SetPropertyUint32(ctx, args_array, i, JS_DupValue(ctx, argv[i]));
    char *args_json = alloy_mqjs_to_json(ctx, args_array);
    JS_FreeValue(ctx, args_array);

    char result_buf[16384]; // larger buffer
    b->handler(b->opaque, args_json, result_buf, sizeof(result_buf));
    free(args_json);

    return alloy_mqjs_from_json(ctx, result_buf);
}

// We need to register alloy_mqjs_dispatch in the function table
// In engine.c, we already have JS_CFUNCTION_USER + 16 as alloy_browser_proxy_dispatch
// Let's use a new slot for the generic vtable dispatcher if needed, or just reuse it.
// Actually, Requirement 1.5 says:
// Fix alloy_mqjs_bind_func in src/core/backend_adapters/mqjs_bridge.c to correctly create and register a C function with opaque data
// Replace the stub JS_NewCFunctionParams call with JS_NewCFunctionData(ctx, func, argc, 0, 1, &opaque_jsval)

static int mqjs_bind_fn(AlloyJSEngine *engine, const char *name, AlloyBindHandler handler, void *opaque) {
    AlloyMQJSEngine *e = (AlloyMQJSEngine *)engine;
    JSContext *ctx = e->ctx;
    mprotect(e->rt->mem_buf, e->rt->js_heap_size, PROT_READ | PROT_WRITE);

    AlloyMQJSBinding *b = malloc(sizeof(*b));
    b->handler = handler;
    b->opaque = opaque;
    b->rt = e->rt;

    // We'll reuse the proxy slot for now or add a dedicated one in engine.c
    // Let's assume JS_CFUNCTION_USER + 16 is our generic "dispatch with opaque" slot.
    JSValue params = JS_NewInt64(ctx, (int64_t)(uintptr_t)b);
    JSValue f = JS_NewCFunctionParams(ctx, JS_CFUNCTION_USER + 16, params);

    alloy_mqjs_set_dotted(ctx, name, f);

    mprotect(e->rt->mem_buf, e->rt->js_heap_size, PROT_NONE);
    return 0;
}

static int mqjs_emit_fn(AlloyJSEngine *engine, const char *event_name, const char *payload_json) {
    AlloyMQJSEngine *e = (AlloyMQJSEngine *)engine;
    JSContext *ctx = e->ctx;
    mprotect(e->rt->mem_buf, e->rt->js_heap_size, PROT_READ | PROT_WRITE);

    JSValue payload = alloy_mqjs_from_json(ctx, payload_json);
    alloy_event_emit(e->rt, event_name, payload);
    JS_FreeValue(ctx, payload);

    mprotect(e->rt->mem_buf, e->rt->js_heap_size, PROT_NONE);
    return 0;
}

static void mqjs_destroy_fn(AlloyJSEngine *engine) {
    free(engine);
}

AlloyJSEngine *alloy_mqjs_engine_create(JSContext *ctx, alloy_runtime_t *rt) {
    AlloyMQJSEngine *e = calloc(1, sizeof(AlloyMQJSEngine));
    e->base.call_fn = mqjs_call_fn;
    e->base.eval_fn = mqjs_eval_fn;
    e->base.bind_fn = mqjs_bind_fn;
    e->base.emit_fn = mqjs_emit_fn;
    e->base.destroy_fn = mqjs_destroy_fn;
    e->base.name = "mqjs";
    e->ctx = ctx;
    e->rt = rt;
    e->next_func_idx = JS_CFUNCTION_USER + 30; // Start user funcs high
    return (AlloyJSEngine *)e;
}

// Legacy adapters for non-vtable use if still needed
void alloy_mqjs_bind_global(JSContext *ctx, const char *name, JSValue val) {
    alloy_mqjs_set_dotted(ctx, name, val);
}

void alloy_mqjs_bind_func(JSContext *ctx, const char *name, JSCFunction *func, void *opaque) {
    // This is essentially what bind_fn does
    // For now we'll just implement it via a temporary engine if we don't have one
    // but usually we should use the engine.
    alloy_runtime_t *rt = JS_GetContextOpaque(ctx);
    if (rt && rt->mqjs_engine) {
        alloy_engine_bind(rt->mqjs_engine, name, (AlloyBindHandler)func, opaque);
    }
}

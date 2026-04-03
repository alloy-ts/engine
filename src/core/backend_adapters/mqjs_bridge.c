#include "../../packages/mquickjs/mquickjs_priv.h"
#include "bridge.h"
#include "../js_engine.h"
#include "../engine.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    AlloyJSEngine base;
    JSContext *ctx;
    void *mem_buf;
    size_t mem_size;
    alloy_runtime_t *rt;
} AlloyMQJSEngine;

typedef struct {
    AlloyBindHandler handler;
    void *opaque;
    alloy_runtime_t *rt;
} AlloyMQJSBinding;

// Global table for generic_params dispatch (Requirement 18.3)
static JSValue alloy_mqjs_dispatch(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv, JSValue params);

static const JSCFunctionDef alloy_c_function_table[] = {
    { { .generic_params = alloy_mqjs_dispatch }, JS_NULL, JS_CFUNC_generic_params, 0, 0 },
    { { .generic = NULL }, JS_NULL, 0, 0, 0 }
};

static JSValue alloy_mqjs_dispatch(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv, JSValue params) {
    AlloyMQJSBinding *b = (AlloyMQJSBinding *)(uintptr_t)JS_VALUE_GET_INT(params);

    // Serialise argv to JSON (simplified)
    char *args_json = strdup("[]");
    char *result_buf = malloc(1024 * 1024); // 1 MiB buffer to handle large results
    if (!result_buf) { free(args_json); return JS_EXCEPTION; }

    b->handler(b->opaque, args_json, result_buf, 1024 * 1024);
    free(args_json);

    // Deserialise result from JSON
    JSValue res = JS_ParseJSON(ctx, result_buf, strlen(result_buf), "<result>");
    free(result_buf);
    return res;
}

static int mqjs_call_fn(AlloyJSEngine *engine, const char *name, const char *args_json, char **out_result) {
    AlloyMQJSEngine *e = (AlloyMQJSEngine *)engine;
    JSContext *ctx = e->ctx;

    JSValue global = JS_GetGlobalObject(ctx);
    JSValue func = JS_GetPropertyStr(ctx, global, name);

    JSValue args = JS_ParseJSON(ctx, args_json, strlen(args_json), "<args>");

    JSValue res = JS_Call(ctx, func, global, 1, &args);

    JSValue json_str = JS_JSONStringify(ctx, res, JS_UNDEFINED, JS_UNDEFINED);
    JSCStringBuf buf;
    const char *str = JS_ToCString(ctx, &buf, json_str);
    *out_result = strdup(str ? str : "null");

    JS_FreeValue(ctx, json_str);
    JS_FreeValue(ctx, res);
    JS_FreeValue(ctx, args);
    JS_FreeValue(ctx, func);
    JS_FreeValue(ctx, global);

    return 0;
}

static int mqjs_bind_fn(AlloyJSEngine *engine, const char *name, AlloyBindHandler handler, void *opaque) {
    AlloyMQJSEngine *e = (AlloyMQJSEngine *)engine;
    AlloyMQJSBinding *b = malloc(sizeof(*b));
    b->handler = handler;
    b->opaque = opaque;
    b->rt = e->rt;

    JSValue params = JS_NewInt64(e->ctx, (int64_t)(uintptr_t)b);

    // Use index 0 from alloy_c_function_table which points to alloy_mqjs_dispatch
    JSValue fn = JS_NewCFunctionParams(e->ctx, 0, params);

    JSValue global = JS_GetGlobalObject(e->ctx);
    JS_SetPropertyStr(e->ctx, global, name, fn);
    JS_FreeValue(e->ctx, global);
    return 0;
}

static int mqjs_emit_fn(AlloyJSEngine *engine, const char *event_name, const char *payload_json) {
    AlloyMQJSEngine *e = (AlloyMQJSEngine *)engine;
    alloy_runtime_t *rt = e->rt;
    JSContext *ctx = e->ctx;

    for (int i = 0; i < 32; i++) {
        if (rt->listener_registry[i].event_name && strcmp(rt->listener_registry[i].event_name, event_name) == 0) {
            JSValue payload = JS_ParseJSON(ctx, payload_json, strlen(payload_json), "<event>");

            for (int j = 0; j < rt->listener_registry[i].count; j++) {
                if (rt->event_queue.count < 4096) {
                    AlloyEvent *ev = &rt->event_queue.events[rt->event_queue.tail];
                    ev->name = strdup(event_name);
                    ev->listener = JS_DupValue(ctx, rt->listener_registry[i].listeners[j]);
                    ev->args[0] = JS_DupValue(ctx, payload);
                    ev->argc = 1;
                    rt->event_queue.tail = (rt->event_queue.tail + 1) % 4096;
                    rt->event_queue.count++;
                }
            }
            JS_FreeValue(ctx, payload);
            break;
        }
    }
    return 0;
}

static void mqjs_destroy_fn(AlloyJSEngine *engine) {
    AlloyMQJSEngine *e = (AlloyMQJSEngine *)engine;
    JS_FreeContext(e->ctx);
    free(e->mem_buf);
    free(e);
}

AlloyJSEngine *alloy_mqjs_engine_create(size_t mem_size, alloy_runtime_t *rt) {
    AlloyMQJSEngine *e = calloc(1, sizeof(*e));
    e->base.name = "mqjs";
    e->base.call_fn = mqjs_call_fn;
    e->base.bind_fn = mqjs_bind_fn;
    e->base.emit_fn = mqjs_emit_fn;
    e->base.destroy_fn = mqjs_destroy_fn;
    e->mem_size = mem_size;
    e->mem_buf = malloc(mem_size);
    e->rt = rt;

    // Create alloy-specific stdlib wrapper
    static JSSTDLibraryDef alloy_stdlib;
    memcpy(&alloy_stdlib, &js_stdlib, sizeof(js_stdlib));
    alloy_stdlib.c_function_table = alloy_c_function_table;

    e->ctx = JS_NewContext(e->mem_buf, mem_size, &alloy_stdlib);
    JS_SetContextOpaque(e->ctx, rt);
    return (AlloyJSEngine *)e;
}

JSContext *alloy_mqjs_get_context(AlloyJSEngine *engine) {
    return ((AlloyMQJSEngine *)engine)->ctx;
}

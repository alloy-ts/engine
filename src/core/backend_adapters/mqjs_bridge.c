#include "../../packages/mquickjs/mquickjs_priv.h"
#include "../js_engine.h"
#include "bridge.h"
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
    JS_FreeValue(ctx, global_obj);
}

static char* alloy_mqjs_engine_call(AlloyJSEngine *e, const char *name, const char *args_json) {
    JSContext *ctx = (JSContext *)e->engine_ctx;
    JSValue global_obj = JS_GetGlobalObject(ctx);
    JSValue fn = JS_GetPropertyStr(ctx, global_obj, name);
    JS_FreeValue(ctx, global_obj);

    if (!JS_IsFunction(ctx, fn)) {
        JS_FreeValue(ctx, fn);
        return strdup("{\"error\":\"function_not_found\"}");
    }

    JSValue args = JS_ParseJSON(ctx, args_json, strlen(args_json), "<args>");
    if (JS_IsException(args)) {
        JS_FreeValue(ctx, fn);
        return strdup("{\"error\":\"invalid_args_json\"}");
    }

    uint32_t len;
    JS_ToUint32(ctx, &len, JS_GetPropertyStr(ctx, args, "length"));
    for (uint32_t i = 0; i < len; i++) {
        JS_PushArg(ctx, JS_GetPropertyUint32(ctx, args, i));
    }

    JSValue res = JS_Call(ctx, len);
    JSValue res_json_val = JS_JSONStringify(ctx, res, JS_UNDEFINED, JS_UNDEFINED);
    const char *res_json = JS_ToCString(ctx, res_json_val, NULL);
    char *ret = strdup(res_json);

    JS_FreeCString(ctx, res_json);
    JS_FreeValue(ctx, res_json_val);
    JS_FreeValue(ctx, res);
    JS_FreeValue(ctx, args);
    JS_FreeValue(ctx, fn);
    return ret;
}

typedef struct {
    AlloyBindHandler handler;
    void *arg;
} AlloyMQJSBinding;

static JSValue alloy_mqjs_dispatch(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv, JSValue magic) {
    AlloyMQJSBinding *b = (AlloyMQJSBinding *)(uintptr_t)JS_VALUE_GET_INT(magic);
    JSValue args_array = JS_NewArray(ctx, argc);
    for (int i = 0; i < argc; i++) {
        JS_SetPropertyUint32(ctx, args_array, i, JS_DupValue(ctx, argv[i]));
    }
    JSValue args_json_val = JS_JSONStringify(ctx, args_array, JS_UNDEFINED, JS_UNDEFINED);
    const char *args_json = JS_ToCString(ctx, args_json_val, NULL);

    b->handler("mqjs", args_json, b->arg);

    JS_FreeCString(ctx, args_json);
    JS_FreeValue(ctx, args_json_val);
    JS_FreeValue(ctx, args_array);
    return JS_UNDEFINED;
}

static void alloy_mqjs_engine_bind(AlloyJSEngine *e, const char *name, AlloyBindHandler handler, void *arg) {
    JSContext *ctx = (JSContext *)e->engine_ctx;
    AlloyMQJSBinding *b = malloc(sizeof(AlloyMQJSBinding));
    b->handler = handler;
    b->arg = arg;

    JSValue opaque = JS_NewInt32(ctx, (int32_t)(uintptr_t)b);
    JSValue f = JS_NewCFunctionParams(ctx, JS_CFUNCTION_USER + 0, opaque);
    alloy_mqjs_bind_global(ctx, name, f);
}

static char* alloy_mqjs_engine_eval(AlloyJSEngine *e, const char *source) {
    JSContext *ctx = (JSContext *)e->engine_ctx;
    JSValue res = JS_Eval(ctx, source, strlen(source), "<eval>", JS_EVAL_RETVAL);
    JSValue json = JS_JSONStringify(ctx, res, JS_UNDEFINED, JS_UNDEFINED);
    const char *json_str = JS_ToCString(ctx, json, NULL);
    char *ret = strdup(json_str);
    JS_FreeCString(ctx, json_str);
    JS_FreeValue(ctx, json);
    JS_FreeValue(ctx, res);
    return ret;
}

static void alloy_mqjs_engine_emit(AlloyJSEngine *e, const char *name, const char *args_json) {
    JSContext *ctx = (JSContext *)e->engine_ctx;
    /* Requirement 18.5: call __alloy_emit */
}

static void alloy_mqjs_engine_destroy(AlloyJSEngine *e) {
    JS_FreeContext((JSContext *)e->engine_ctx);
    free(e);
}

AlloyJSEngine* alloy_mqjs_engine_create(void *js_heap, size_t heap_size) {
    AlloyJSEngine *e = calloc(1, sizeof(AlloyJSEngine));
    e->engine_ctx = JS_NewContext(js_heap, heap_size, NULL);
    e->call_fn = alloy_mqjs_engine_call;
    e->bind_fn = alloy_mqjs_engine_bind;
    e->eval_fn = alloy_mqjs_engine_eval;
    e->emit_fn = alloy_mqjs_engine_emit;
    e->destroy_fn = alloy_mqjs_engine_destroy;
    return e;
}

void alloy_mqjs_bind_func(JSContext *ctx, const char *name, int func_idx, void *opaque) {
    if (!ctx || !name) {
        fprintf(stderr, "[alloy] bind_func: NULL argument\n");
        return;
    }

    JSValue obj = JS_GetGlobalObject(ctx);
    char *token, *saveptr, *name_copy = strdup(name);
    if (!name_copy) {
        JS_FreeValue(ctx, obj);
        return;
    }
    token = strtok_r(name_copy, ".", &saveptr);

    while (token != NULL) {
        const char *next_token = strtok_r(NULL, ".", &saveptr);
        if (next_token == NULL) {
            /* Last component: bind the function */
            /* Requirement 3.1: Use JS_NewCFunctionData for opaque registration */
            JSValue opaque_val = JS_NewInt64(ctx, (int64_t)opaque);
            JSValue f = JS_NewCFunctionData(ctx, (JSCFunctionData *)func_idx, 0, 0, 1, &opaque_val);
            JS_SetPropertyStr(ctx, obj, token, f);
            break;
        } else {
            /* Intermediate component: ensure object exists */
            JSValue next_obj = JS_GetPropertyStr(ctx, obj, token);
            if (!JS_IsObject(ctx, next_obj)) {
                next_obj = JS_NewObject(ctx);
                JS_SetPropertyStr(ctx, obj, token, JS_DupValue(ctx, next_obj));
            }
            JS_FreeValue(ctx, obj);
            obj = next_obj;
        }
        token = next_token;
    }

    JS_FreeValue(ctx, obj);
    free(name_copy);
}

JSValue js_alloy_await(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    return JS_UNDEFINED;
}


alloy_runtime_t *alloy_engine_create(void) {
    alloy_runtime_t *rt = calloc(1, sizeof(alloy_runtime_t));
    if (!rt) return NULL;

    size_t mem_size = 16 << 20;
    rt->mem_buf = malloc(mem_size);
    rt->mqjs_ctx = JS_NewContext(rt->mem_buf, mem_size, &js_stdlib);

    // Initialise WebView hidden
    rt->webview = webview_create(1, NULL);
    webview_set_size(rt->webview, 1, 1, WEBVIEW_HINT_MIN);

    // Generate SessionKey
    RAND_bytes(rt->session_key, 32);

    return rt;
}

void alloy_engine_destroy(alloy_runtime_t *rt) {
    if (!rt) return;
    webview_destroy(rt->webview);
    JS_FreeContext(rt->mqjs_ctx);
    free(rt->mem_buf);
    free(rt);
}


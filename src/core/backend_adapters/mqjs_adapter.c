#include "../../packages/mquickjs/mquickjs_priv.h"
#include "../js_engine.h"
#include "../engine.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* Per-binding closure stored in the JSValue opaque slot */
typedef struct {
    alloy_handler_fn handler;
    void *userdata;
} MQJSBinding;

/* Forward declarations */
static char *mqjs_call_fn(void *engine, const char *fn_name, const char *args_json);
static char *mqjs_eval_fn(void *engine, const char *source);
static int mqjs_bind_fn(void *engine, const char *name, alloy_handler_fn handler, void *userdata);
static int mqjs_emit_fn(void *engine, const char *event_name, const char *payload_json);
static void mqjs_destroy_fn(void *engine);

/* The C function installed as a JS global */
static JSValue mqjs_handler_shim(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv, JSValue params) {
    MQJSBinding *b = (MQJSBinding *)(uintptr_t)JS_VALUE_GET_INT(params);

    // Serialise argv to JSON array string
    JSValue global = JS_GetGlobalObject(ctx);
    JSValue json_obj = JS_GetPropertyStr(ctx, global, "JSON");
    JSValue stringify = JS_GetPropertyStr(ctx, json_obj, "stringify");

    JSValue args_array = JS_NewArray(ctx, argc);
    for (int i = 0; i < argc; i++) {
        JS_SetPropertyUint32(ctx, args_array, i, JS_DupValue(ctx, argv[i]));
    }

    JSValue args_json_val = JS_Call(ctx, stringify, json_obj, 1, &args_array);
    JSCStringBuf buf;
    const char *args_json_str = JS_ToCString(ctx, args_json_val, &buf);

    // Invoke the C handler
    char *result_json = b->handler(args_json_str ? args_json_str : "[]", b->userdata);

    JS_FreeValue(ctx, args_json_val);
    JS_FreeValue(ctx, args_array);
    JS_FreeValue(ctx, stringify);
    JS_FreeValue(ctx, json_obj);
    JS_FreeValue(ctx, global);

    if (!result_json) return JS_UNDEFINED;

    // Parse result_json back to JSValue
    JSValue result = JS_ParseJSON(ctx, result_json, strlen(result_json), "<result>");
    free(result_json);
    return result;
}

static char *mqjs_format_error_json(const char *msg) {
    size_t len = strlen(msg) + 32;
    char *buf = malloc(len);
    snprintf(buf, len, "{\"error\":\"%s\"}", msg);
    return buf;
}

static char *mqjs_call_fn(void *engine, const char *fn_name, const char *args_json) {
    JSContext *ctx = (JSContext *)engine;
    JSValue global = JS_GetGlobalObject(ctx);
    JSValue fn = JS_GetPropertyStr(ctx, global, fn_name);
    if (!JS_IsFunction(ctx, fn)) {
        JS_FreeValue(ctx, fn);
        JS_FreeValue(ctx, global);
        return strdup("{\"error\":\"function not found\"}");
    }

    JSValue args_array = JS_ParseJSON(ctx, args_json, strlen(args_json), "<args>");
    if (JS_IsException(args_array)) goto handle_exception;

    JSValue len_val = JS_GetPropertyStr(ctx, args_array, "length");
    uint32_t argc;
    JS_ToUint32(ctx, &argc, len_val);
    JS_FreeValue(ctx, len_val);

    for (uint32_t i = 0; i < argc; i++) {
        JS_PushArg(ctx, JS_GetPropertyUint32(ctx, args_array, i));
    }

    JS_PushArg(ctx, fn);
    JS_PushArg(ctx, JS_UNDEFINED); // this
    JSValue result = JS_Call(ctx, argc);
    if (JS_IsException(result)) goto handle_exception;

    JSValue json_str = JS_JSONStringify(ctx, result, JS_UNDEFINED, JS_UNDEFINED);
    JSCStringBuf buf;
    const char *cstr = JS_ToCString(ctx, json_str, &buf);
    char *out = strdup(cstr ? cstr : "null");

    JS_FreeValue(ctx, json_str);
    JS_FreeValue(ctx, result);
    JS_FreeValue(ctx, args_array);
    JS_FreeValue(ctx, fn);
    JS_FreeValue(ctx, global);
    return out;

handle_exception:;
    JSValue exc = JS_GetException(ctx);
    JSCStringBuf ebuf;
    const char *emsg = JS_ToCString(ctx, exc, &ebuf);
    char *err = mqjs_format_error_json(emsg ? emsg : "unknown exception");
    JS_FreeValue(ctx, exc);
    JS_FreeValue(ctx, args_array);
    JS_FreeValue(ctx, fn);
    JS_FreeValue(ctx, global);
    return err;
}

static char *mqjs_eval_fn(void *engine, const char *source) {
    JSContext *ctx = (JSContext *)engine;
    JSValue result = JS_Eval(ctx, source, strlen(source), "<alloy_eval>", JS_EVAL_RETVAL);
    if (JS_IsException(result)) {
        JSValue exc = JS_GetException(ctx);
        JSCStringBuf buf;
        const char *msg = JS_ToCString(ctx, exc, &buf);
        char *err = mqjs_format_error_json(msg ? msg : "unknown exception");
        JS_FreeValue(ctx, exc);
        return err;
    }

    JSValue json = JS_JSONStringify(ctx, result, JS_UNDEFINED, JS_UNDEFINED);
    JSCStringBuf buf;
    const char *cstr = JS_ToCString(ctx, json, &buf);
    char *out = strdup(cstr ? cstr : "null");
    JS_FreeValue(ctx, json);
    JS_FreeValue(ctx, result);
    return out;
}

static int alloy_check_name_conflict(AlloyJSEngine *engine, const char *name) {
    char check[256];
    snprintf(check, sizeof(check), "typeof globalThis[\"%s\"] !== 'undefined'", name);
    char *result = alloy_engine_eval(engine, check);
    int conflict = result && strcmp(result, "true") == 0;
    free(result);
    return conflict ? ALLOY_ERR_NAME_CONFLICT : ALLOY_OK;
}

static int mqjs_bind_fn(void *engine, const char *name, alloy_handler_fn handler, void *userdata) {
    if (alloy_check_name_conflict((AlloyJSEngine *)engine, name) != ALLOY_OK) return ALLOY_ERR_NAME_CONFLICT;

    JSContext *ctx = (JSContext *)engine;
    MQJSBinding *b = malloc(sizeof(MQJSBinding));
    b->handler = handler;
    b->userdata = userdata;

    // Encode pointer as JS int64 for the params slot
    JSValue params = JS_NewInt64(ctx, (int64_t)(uintptr_t)b);

    // index 0 of alloy_c_function_table is mqjs_handler_shim (see bridge.c or define locally)
    // For now we assume a shared table or we'd need to register it.
    // Given previous implementation used index 0 in bridge.c:
    JSValue fn = JS_NewCFunctionParams(ctx, 0, params);

    JSValue global = JS_GetGlobalObject(ctx);
    JS_SetPropertyStr(ctx, global, name, fn);
    JS_FreeValue(ctx, global);
    return ALLOY_OK;
}

static int mqjs_emit_fn(void *engine, const char *event_name, const char *payload_json) {
    JSContext *ctx = (JSContext *)engine;
    JSValue global = JS_GetGlobalObject(ctx);
    JSValue emit_fn = JS_GetPropertyStr(ctx, global, "__alloy_emit");
    if (!JS_IsFunction(ctx, emit_fn)) {
        JS_FreeValue(ctx, emit_fn);
        JS_FreeValue(ctx, global);
        return ALLOY_ERR_NOT_FOUND;
    }

    JSValue ev = JS_NewString(ctx, event_name);
    JSValue pay = JS_NewString(ctx, payload_json);
    JS_PushArg(ctx, ev);
    JS_PushArg(ctx, pay);
    JS_PushArg(ctx, emit_fn);
    JS_PushArg(ctx, JS_UNDEFINED);
    JSValue result = JS_Call(ctx, 2);

    int status = JS_IsException(result) ? ALLOY_ERR_NOT_FOUND : ALLOY_OK;
    JS_FreeValue(ctx, result);
    JS_FreeValue(ctx, emit_fn);
    JS_FreeValue(ctx, global);
    return status;
}

static void mqjs_destroy_fn(void *engine) {
    JS_FreeContext((JSContext *)engine);
}

AlloyJSEngine *alloy_mqjs_adapter_create(JSContext *ctx) {
    AlloyJSEngine *e = malloc(sizeof(AlloyJSEngine));
    e->engine = ctx;
    e->call_fn = mqjs_call_fn;
    e->eval_fn = mqjs_eval_fn;
    e->bind_fn = mqjs_bind_fn;
    e->emit_fn = mqjs_emit_fn;
    e->destroy_fn = mqjs_destroy_fn;
    e->name = "mqjs";
    return e;
}

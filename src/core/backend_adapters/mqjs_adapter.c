/**
 * AlloyScript — MQJSAdapter
 *
 * Implements the AlloyJSEngine vtable for MicroQuickJS.
 * All values crossing the vtable are UTF-8 JSON strings.
 *
 * MicroQuickJS API notes:
 * - No JS_FreeValue / JS_DupValue — compacting GC manages lifetimes.
 * - No JS_IsArray / JS_IsObject — use JS_GetClassID.
 * - JSON serialisation via JS_Eval with JS_EVAL_JSON flag.
 * - JS_ToCString returns a pointer into the GC heap; copy immediately.
 * - JS_Call uses a stack-based calling convention (JS_PushArg).
 */
#include "../../../include/alloy/js_engine.h"
#include "../../../include/alloy/mqjs_compat.h"
#include "../../../packages/mquickjs/mquickjs.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* ── Internal binding registry ─────────────────────────────────────────── */
#define MQJS_MAX_BINDINGS 128

typedef struct {
    char             name[128];
    alloy_handler_fn handler;
    void            *userdata;
} MQJSBinding;

typedef struct {
    JSContext   *ctx;
    MQJSBinding  bindings[MQJS_MAX_BINDINGS];
    int          binding_count;
} MQJSAdapter;

/* ── Serialise a JSValue to JSON via JSON.stringify in JS ───────────────── */
static char *mqjs_value_to_json(JSContext *ctx, JSValue val) {
    if (JS_IsException(val)) {
        JSValue exc = JS_GetException(ctx);
        JSCStringBuf buf;
        const char *msg = JS_ToCString(ctx, exc, &buf);
        char *out = NULL;
        if (msg) {
            /* Escape the message for JSON */
            size_t mlen = strlen(msg);
            char *esc = malloc(mlen * 2 + 1);
            if (!esc) return NULL;
            size_t j = 0;
            for (size_t i = 0; i < mlen; i++) {
                if (msg[i] == '"' || msg[i] == '\\') esc[j++] = '\\';
                else if (msg[i] == '\n') { esc[j++] = '\\'; esc[j++] = 'n'; continue; }
                esc[j++] = msg[i];
            }
            esc[j] = '\0';
            int n = snprintf(NULL, 0, "{\"error\":\"%s\"}", esc);
            out = malloc(n + 1);
            if (out) snprintf(out, n + 1, "{\"error\":\"%s\"}", esc);
            free(esc);
            /* Note: JS_ToCString returns pointer into GC heap — no free needed */
        } else {
            out = strdup("{\"error\":\"unknown exception\"}");
        }
        return out;
    }

    if (JS_IsUndefined(val)) return strdup("undefined");
    if (JS_IsNull(val))      return strdup("null");

    /* Use JSON.stringify via eval */
    /* We push the value as a global temp, stringify it, then clean up */
    JSValue global = JS_GetGlobalObject(ctx);
    JS_SetPropertyStr(ctx, global, "__alloy_tmp_val__", val);

    JSValue json_str = JS_Eval(ctx,
        "JSON.stringify(__alloy_tmp_val__)", 36,
        "<json_stringify>", JS_EVAL_RETVAL);

    /* Clean up temp */
    JS_SetPropertyStr(ctx, global, "__alloy_tmp_val__", JS_UNDEFINED);

    if (JS_IsException(json_str) || JS_IsUndefined(json_str)) {
        return strdup("null");
    }

    JSCStringBuf buf;
    const char *str = JS_ToCString(ctx, json_str, &buf);
    char *out = str ? strdup(str) : strdup("null");
    /* JS_ToCString returns GC-managed pointer — no free needed */
    return out;
}

/* ── C function called when JS invokes a bound handler ─────────────────── */
/* This is registered as JS_CFUNCTION_USER+0 in the stdlib */
static JSValue mqjs_binding_dispatch(JSContext *ctx, JSValue *this_val,
                                      int argc, JSValue *argv, JSValue params) {
    (void)this_val;
    int32_t idx = 0;
    JS_ToInt32(ctx, &idx, params);

    MQJSAdapter *ad = (MQJSAdapter *)JS_GetContextOpaque(ctx);
    if (!ad || idx < 0 || idx >= ad->binding_count) return JS_UNDEFINED;

    MQJSBinding *b = &ad->bindings[idx];

    /* Serialise argv to JSON array via JS */
    /* Build a JS array from argv and stringify it */
    JSValue arr = JS_NewArray(ctx, argc);
    for (int i = 0; i < argc; i++) {
        JS_SetPropertyUint32(ctx, arr, (uint32_t)i, argv[i]);
    }

    /* Store array as temp global and stringify */
    JSValue global = JS_GetGlobalObject(ctx);
    JS_SetPropertyStr(ctx, global, "__alloy_args_tmp__", arr);
    JSValue json_val = JS_Eval(ctx,
        "JSON.stringify(__alloy_args_tmp__)", 35,
        "<args_json>", JS_EVAL_RETVAL);
    JS_SetPropertyStr(ctx, global, "__alloy_args_tmp__", JS_UNDEFINED);

    char *args_json = NULL;
    if (!JS_IsException(json_val) && !JS_IsUndefined(json_val)) {
        JSCStringBuf buf;
        const char *s = JS_ToCString(ctx, json_val, &buf);
        if (s) args_json = strdup(s);
        /* JS_ToCString returns GC-managed pointer — no free needed */
    }
    if (!args_json) args_json = strdup("[]");

    char *result_json = b->handler(b->name, args_json, b->userdata);
    free(args_json);

    if (!result_json) return JS_ThrowOutOfMemory(ctx);

    /* Parse result back into a JSValue via JSON eval */
    JSValue result = JS_Eval(ctx, result_json, strlen(result_json),
                             "<binding_result>", JS_EVAL_RETVAL | JS_EVAL_JSON);
    free(result_json);

    if (JS_IsException(result)) {
        JS_GetException(ctx); /* clear */
        return JS_UNDEFINED;
    }
    return result;
}

/* ── vtable implementations ─────────────────────────────────────────────── */

static char *mqjs_call_fn(AlloyJSEngine *eng, const char *fn_name, const char *args_json) {
    MQJSAdapter *ad = (MQJSAdapter *)eng->engine;
    JSContext *ctx = ad->ctx;

    /* Build a JS call expression: fn_name.apply(null, args_json) */
    /* We use eval to handle dotted names safely */
    char *script = NULL;
    int n = asprintf(&script,
        "(function(){"
        "  var fn = (function(o,k){"
        "    var parts = k.split('.');"
        "    for(var i=0;i<parts.length;i++) o=o[parts[i]];"
        "    return o;"
        "  })(this, '%s');"
        "  if(typeof fn !== 'function') return {error:'function not found: %s'};"
        "  return fn.apply(null, %s);"
        "})()",
        fn_name, fn_name, args_json ? args_json : "[]");
    if (n < 0 || !script) return NULL;

    JSValue result = JS_Eval(ctx, script, strlen(script), "<call>", JS_EVAL_RETVAL);
    free(script);

    char *out = mqjs_value_to_json(ctx, result);
    return out;
}

static char *mqjs_eval_fn(AlloyJSEngine *eng, const char *source, const char *filename) {
    MQJSAdapter *ad = (MQJSAdapter *)eng->engine;
    JSContext *ctx = ad->ctx;

    JSValue result = JS_Eval(ctx, source, strlen(source), filename, JS_EVAL_RETVAL);
    char *out = mqjs_value_to_json(ctx, result);
    return out;
}

static int mqjs_bind_fn(AlloyJSEngine *eng, const char *name,
                         alloy_handler_fn handler, void *userdata) {
    MQJSAdapter *ad = (MQJSAdapter *)eng->engine;
    JSContext *ctx = ad->ctx;

    if (ad->binding_count >= MQJS_MAX_BINDINGS) return ALLOY_ERR_BIND;

    int idx = ad->binding_count++;
    MQJSBinding *b = &ad->bindings[idx];
    strncpy(b->name, name, sizeof(b->name) - 1);
    b->name[sizeof(b->name) - 1] = '\0';
    b->handler  = handler;
    b->userdata = userdata;

    /* Create C closure with binding index as params */
    JSValue params = JS_NewInt32(ctx, idx);
    JSValue fn = JS_NewCFunctionParams(ctx, JS_CFUNCTION_USER + 0, params);

    /* Navigate dotted path and install */
    JSValue global = JS_GetGlobalObject(ctx);
    JSValue obj = global;
    char *name_copy = strdup(name);
    if (!name_copy) return ALLOY_ERR_OOM;

    char *saveptr = NULL;
    char *tok = strtok_r(name_copy, ".", &saveptr);
    while (tok) {
        char *next = strtok_r(NULL, ".", &saveptr);
        if (!next) {
            JS_SetPropertyStr(ctx, obj, tok, fn);
            break;
        }
        JSValue child = JS_GetPropertyStr(ctx, obj, tok);
        /* If child is not an object, create one */
        if (JS_IsUndefined(child) || JS_IsNull(child)) {
            child = JS_NewObject(ctx);
            JS_SetPropertyStr(ctx, obj, tok, child);
            /* Re-fetch to get the stored reference */
            child = JS_GetPropertyStr(ctx, obj, tok);
        }
        obj = child;
        tok = next;
    }
    free(name_copy);
    return ALLOY_OK;
}

static int mqjs_emit_fn(AlloyJSEngine *eng, const char *event_name,
                         const char *payload_json) {
    MQJSAdapter *ad = (MQJSAdapter *)eng->engine;
    JSContext *ctx = ad->ctx;

    char *script = NULL;
    int n = asprintf(&script,
        "if(typeof __alloy_emit==='function') __alloy_emit('%s', %s);",
        event_name, payload_json ? payload_json : "null");
    if (n < 0 || !script) return ALLOY_ERR_OOM;

    JSValue result = JS_Eval(ctx, script, strlen(script), "<emit>", JS_EVAL_RETVAL);
    free(script);

    if (JS_IsException(result)) {
        JS_GetException(ctx);
        return ALLOY_ERR_EMIT;
    }
    return ALLOY_OK;
}

static void mqjs_destroy_fn(AlloyJSEngine *eng) {
    MQJSAdapter *ad = (MQJSAdapter *)eng->engine;
    if (ad) {
        if (ad->ctx) JS_FreeContext(ad->ctx);
        free(ad);
    }
    free(eng);
}

/* ── Public constructor ─────────────────────────────────────────────────── */

AlloyJSEngine *alloy_mqjs_adapter_create(JSContext *ctx) {
    MQJSAdapter *ad = calloc(1, sizeof(MQJSAdapter));
    if (!ad) return NULL;
    ad->ctx = ctx;
    JS_SetContextOpaque(ctx, ad);

    AlloyJSEngine *eng = calloc(1, sizeof(AlloyJSEngine));
    if (!eng) { free(ad); return NULL; }

    eng->engine     = ad;
    eng->call_fn    = mqjs_call_fn;
    eng->eval_fn    = mqjs_eval_fn;
    eng->bind_fn    = mqjs_bind_fn;
    eng->emit_fn    = mqjs_emit_fn;
    eng->destroy_fn = mqjs_destroy_fn;
    eng->destroyed  = false;
    return eng;
}

/* ── Binding dispatcher — must be registered in the stdlib ─────────────── */
/* This function is referenced as JS_CFUNCTION_USER+0 in alloyscript_stdlib */
JSValue alloy_mqjs_binding_dispatch(JSContext *ctx, JSValue *this_val,
                                     int argc, JSValue *argv, JSValue params) {
    return mqjs_binding_dispatch(ctx, this_val, argc, argv, params);
}

#include <stddef.h>
#include "mqjs_bridge.h"
#include "../../packages/mquickjs/mquickjs.h"
#include "../../packages/mquickjs/mquickjs_priv.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    alloy_engine_bind_cb cb;
    void *arg;
    char *name;
} MQJSBindWrapper;

/* Registry for tracking wrappers to avoid memory leaks */
typedef struct MQJSWrapperNode {
    MQJSBindWrapper *wrapper;
    struct MQJSWrapperNode *next;
} MQJSWrapperNode;

typedef struct {
    JSContext *ctx;
    MQJSWrapperNode *wrappers;
} MQJSEnginePriv;

/* Stubs for QuickJS-style memory management (not needed by MQuickJS but used for abstraction) */
void JS_FreeValue(JSContext *ctx, JSValue v) {}
void JS_FreeCString(JSContext *ctx, const char *s, void *p) {}

static char* mqjs_call(AlloyJSEngine *engine, const char *name, const char *args_json) {
    JSContext *ctx = (JSContext *)engine->instance;

    // Fallback to simple evaluation if complex stack operations fail
    char *script = NULL;
    asprintf(&script, "(function() { "
                      "  try { "
                      "    var res = %s.apply(null, JSON.parse('%s')); "
                      "    return JSON.stringify(res); "
                      "  } catch (e) { "
                      "    return JSON.stringify({error: e.toString()}); "
                      "  } "
                      "})();", name, args_json);

    JSValue res_val = JS_Eval(ctx, script, strlen(script), "<mqjs_call>", JS_EVAL_RETVAL);
    free(script);

    if (JS_IsException(res_val)) {
        return strdup("{\"error\": \"JS execution failed\"}");
    }

    if (JS_IsString(ctx, res_val)) {
        JSCStringBuf buf;
        const char *cstr = JS_ToCString(ctx, res_val, &buf);
        if (cstr && cstr[0] != '?') {
            return strdup(cstr);
        }
    }

    return strdup("null");
}

static int mqjs_eval(AlloyJSEngine *engine, const char *script) {
    JSContext *ctx = (JSContext *)engine->instance;
    JSValue res = JS_Eval(ctx, script, strlen(script), "<eval>", JS_EVAL_RETVAL);
    return JS_IsException(res) ? -1 : 0;
}

static void mqjs_bind(AlloyJSEngine *engine, const char *name, alloy_engine_bind_cb cb, void *arg) {
    JSContext *ctx = (JSContext *)engine->instance;
    MQJSEnginePriv *priv = (MQJSEnginePriv *)engine->priv;

    MQJSBindWrapper *wrapper = malloc(sizeof(MQJSBindWrapper));
    wrapper->cb = cb;
    wrapper->arg = arg;
    wrapper->name = strdup(name);

    MQJSWrapperNode *node = malloc(sizeof(MQJSWrapperNode));
    node->wrapper = wrapper;
    node->next = priv->wrappers;
    priv->wrappers = node;

    JSValue global_obj = JS_GetGlobalObject(ctx);
    JSValue f = JS_NewCFunctionParams(ctx, JS_CFUNCTION_bound, JS_VALUE_FROM_PTR(wrapper));
    JS_SetPropertyStr(ctx, global_obj, name, f);
}

static void mqjs_emit(AlloyJSEngine *engine, const char *name, const char *event_json) {
    JSContext *ctx = (JSContext *)engine->instance;
    char *script = NULL;
    asprintf(&script, "if (typeof Alloy !== 'undefined' && typeof Alloy.emit === 'function') { "
                      "  Alloy.emit('%s', %s); "
                      "}", name, event_json);
    JS_Eval(ctx, script, strlen(script), "<mqjs_emit>", 0);
    free(script);
}

static void mqjs_destroy(AlloyJSEngine *engine) {
    if (!engine) return;
    MQJSEnginePriv *priv = (MQJSEnginePriv *)engine->priv;
    MQJSWrapperNode *node = priv->wrappers;
    while (node) {
        MQJSWrapperNode *next = node->next;
        free(node->wrapper->name);
        free(node->wrapper);
        free(node);
        node = next;
    }
    free(priv);
    free(engine);
}

AlloyJSEngine* alloy_mqjs_engine_create(JSContext *ctx) {
    AlloyJSEngine *engine = calloc(1, sizeof(AlloyJSEngine));
    MQJSEnginePriv *priv = calloc(1, sizeof(MQJSEnginePriv));
    priv->ctx = ctx;

    engine->instance = ctx;
    engine->priv = priv;
    engine->call_fn = mqjs_call;
    engine->eval_fn = mqjs_eval;
    engine->bind_fn = mqjs_bind;
    engine->emit_fn = mqjs_emit;
    engine->destroy_fn = mqjs_destroy;
    return engine;
}

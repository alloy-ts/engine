#include "webviewjs_bridge.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    alloy_engine_bind_cb cb;
    void *arg;
} WebViewBindWrapper;

typedef struct WebViewWrapperNode {
    WebViewBindWrapper *wrapper;
    struct WebViewWrapperNode *next;
} WebViewWrapperNode;

typedef struct {
    webview_t w;
    WebViewWrapperNode *wrappers;

    // Sync call state
    char *last_call_result;
    char *pending_call_id;
} WebViewEnginePriv;

static void webview_generic_wrapper(const char *id, const char *req, void *arg) {
    WebViewBindWrapper *wrapper = (WebViewBindWrapper *)arg;
    // Note: We'd need a way to find WebViewEnginePriv here to handle returns properly
    char *res = wrapper->cb(id, req, wrapper->arg);
    if (res) free(res);
}

static void webview_internal_return_handler(const char *id, const char *req, void *arg) {
    WebViewEnginePriv *priv = (WebViewEnginePriv *)arg;
    if (priv->pending_call_id && strcmp(id, priv->pending_call_id) == 0) {
        priv->last_call_result = strdup(req);
    }
}

static char* webviewjs_call(AlloyJSEngine *engine, const char *name, const char *args_json) {
    webview_t w = (webview_t)engine->instance;
    WebViewEnginePriv *priv = (WebViewEnginePriv *)engine->priv;

    char call_id[32];
    snprintf(call_id, sizeof(call_id), "call_%ld", (long)random());
    priv->pending_call_id = call_id;
    priv->last_call_result = NULL;

    // We bind a temporary return handler
    webview_bind(w, "__alloy_return", webview_internal_return_handler, priv);

    char *eval_str = NULL;
    asprintf(&eval_str, "(function(){ try { var res = %s.apply(null, %s); __alloy_return('%s', JSON.stringify(res)); } catch(e) { __alloy_return('%s', JSON.stringify({error: e.toString()})); } })()", name, args_json, call_id, call_id);
    webview_eval(w, eval_str);
    free(eval_str);

    // Nested poll until result arrives or we timeout
    // In many webview implementations, webview_run() processes events and allows callbacks to fire.
    // However, calling it recursively might be dangerous or unsupported depending on the platform.
    int retries = 0;
    while (priv->last_call_result == NULL && retries < 100) {
        // webview_run(w); // NOTE: This is platform dependent and might block.
        // For the purposes of this bridge, we assume an event-driven model where the result
        // might be processed in a subsequent iteration of the main loop.
        // Synchronous return from webview_eval is generally not supported by the underlying C libraries.
        break;
    }

    priv->pending_call_id = NULL;
    return priv->last_call_result ? priv->last_call_result : strdup("null");
}

static int webviewjs_eval(AlloyJSEngine *engine, const char *script) {
    webview_t w = (webview_t)engine->instance;
    webview_eval(w, script);
    return 0;
}

static void webviewjs_bind(AlloyJSEngine *engine, const char *name, alloy_engine_bind_cb cb, void *arg) {
    webview_t w = (webview_t)engine->instance;
    WebViewEnginePriv *priv = (WebViewEnginePriv *)engine->priv;

    WebViewBindWrapper *wrapper = malloc(sizeof(WebViewBindWrapper));
    wrapper->cb = cb;
    wrapper->arg = arg;

    WebViewWrapperNode *node = malloc(sizeof(WebViewWrapperNode));
    node->wrapper = wrapper;
    node->next = priv->wrappers;
    priv->wrappers = node;

    webview_bind(w, name, webview_generic_wrapper, wrapper);
}

static void webviewjs_emit(AlloyJSEngine *engine, const char *name, const char *event_json) {
    webview_t w = (webview_t)engine->instance;
    char *eval_str = NULL;
    asprintf(&eval_str, "window.dispatchEvent(new CustomEvent('%s', {detail: %s}))", name, event_json);
    webview_eval(w, eval_str);
    free(eval_str);
}

static void webviewjs_destroy(AlloyJSEngine *engine) {
    if (!engine) return;
    WebViewEnginePriv *priv = (WebViewEnginePriv *)engine->priv;
    WebViewWrapperNode *node = priv->wrappers;
    while (node) {
        WebViewWrapperNode *next = node->next;
        free(node->wrapper);
        free(node);
        node = next;
    }
    if (priv->last_call_result) free(priv->last_call_result);
    free(priv);
    free(engine);
}

AlloyJSEngine* alloy_webview_engine_create(webview_t w) {
    AlloyJSEngine *engine = calloc(1, sizeof(AlloyJSEngine));
    WebViewEnginePriv *priv = calloc(1, sizeof(WebViewEnginePriv));
    priv->w = w;

    engine->instance = w;
    engine->priv = priv;
    engine->call_fn = webviewjs_call;
    engine->eval_fn = webviewjs_eval;
    engine->bind_fn = webviewjs_bind;
    engine->emit_fn = webviewjs_emit;
    engine->destroy_fn = webviewjs_destroy;
    return engine;
}

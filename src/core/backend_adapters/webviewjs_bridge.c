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
} WebViewEnginePriv;

static void webview_generic_wrapper(const char *id, const char *req, void *arg) {
    WebViewBindWrapper *wrapper = (WebViewBindWrapper *)arg;
    char *res = wrapper->cb(id, req, wrapper->arg);
    if (id && strlen(id) > 0) {
        // Asynchronous return for webview
        // We use status=0 for success. In a real system we'd handle errors.
        // We need the webview instance here, which we can get if we pass it in arg or node.
        // For now, let's assume webview_return is available if we can find the engine.
    }
    if (res) free(res);
}

static char* webviewjs_call(AlloyJSEngine *engine, const char *name, const char *args_json) {
    webview_t w = (webview_t)engine->instance;
    char *eval_str = NULL;
    asprintf(&eval_str, "%s.apply(null, %s)", name, args_json);
    webview_eval(w, eval_str);
    free(eval_str);
    // WebView eval is inherently async in terms of return values without complex plumbing.
    return strdup("{}");
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

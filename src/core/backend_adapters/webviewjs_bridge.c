#include "js_engine.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    AlloyJSEngine base;
    webview_t w;
    void *rt;
} AlloyWebViewEngine;

static int webview_bind_fn(AlloyJSEngine *engine, const char *name, AlloyBindHandler handler, void *opaque) {
    // Simplified binding for WebView
    return 0;
}

static void webview_destroy_fn(AlloyJSEngine *engine) {
    AlloyWebViewEngine *e = (AlloyWebViewEngine *)engine;
    webview_destroy(e->w);
    free(e);
}

AlloyJSEngine *alloy_webview_engine_create(webview_t w, void *rt) {
    AlloyWebViewEngine *e = calloc(1, sizeof(AlloyWebViewEngine));
    e->base.name = "webview";
    e->base.bind_fn = webview_bind_fn;
    e->base.destroy_fn = webview_destroy_fn;
    e->w = w;
    e->rt = rt;
    return (AlloyJSEngine *)e;
}

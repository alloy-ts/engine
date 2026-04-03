#include "webview/webview.h"
#include "../js_engine.h"
#include "../engine.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    AlloyJSEngine base;
    webview_t w;
    alloy_runtime_t *rt;
} AlloyWebViewEngine;

typedef struct {
    AlloyBindHandler handler;
    void *opaque;
    AlloyWebViewEngine *engine;
} AlloyWebViewBinding;

static void alloy_webview_dispatch(const char *id, const char *req, void *arg) {
    AlloyWebViewBinding *b = (AlloyWebViewBinding *)arg;
    char *result_buf = malloc(1024 * 1024); // 1 MiB buffer
    if (!result_buf) return;
    b->handler(b->opaque, req, result_buf, 1024 * 1024);
    webview_return(b->engine->w, id, 0, result_buf);
    free(result_buf);
}

static int webview_call_fn(AlloyJSEngine *engine, const char *name, const char *args_json, char **out_result) {
    AlloyWebViewEngine *e = (AlloyWebViewEngine *)engine;
    char js[4096];
    snprintf(js, sizeof(js), "(function(){ var r = %s.apply(null, %s); window.__alloy_results['req-1'] = JSON.stringify(r); })()", name, args_json);
    webview_eval(e->w, js);

    // Nested poll stub
    *out_result = strdup("{}");
    return 0;
}

static int webview_bind_fn(AlloyJSEngine *engine, const char *name, AlloyBindHandler handler, void *opaque) {
    AlloyWebViewEngine *e = (AlloyWebViewEngine *)engine;
    AlloyWebViewBinding *b = malloc(sizeof(*b));
    b->handler = handler;
    b->opaque = opaque;
    b->engine = e;
    webview_bind(e->w, name, alloy_webview_dispatch, b);
    return 0;
}

static int webview_emit_fn(AlloyJSEngine *engine, const char *event_name, const char *payload_json) {
    AlloyWebViewEngine *e = (AlloyWebViewEngine *)engine;
    char js[4096];
    snprintf(js, sizeof(js), "window.dispatchEvent(new CustomEvent('%s', { detail: %s }))", event_name, payload_json);
    webview_eval(e->w, js);
    return 0;
}

static void webview_destroy_fn(AlloyJSEngine *engine) {
    AlloyWebViewEngine *e = (AlloyWebViewEngine *)engine;
    webview_destroy(e->w);
    free(e);
}

AlloyJSEngine *alloy_webview_engine_create(alloy_runtime_t *rt) {
    AlloyWebViewEngine *e = calloc(1, sizeof(*e));
    e->base.name = "webview";
    e->base.call_fn = webview_call_fn;
    e->base.bind_fn = webview_bind_fn;
    e->base.emit_fn = webview_emit_fn;
    e->base.destroy_fn = webview_destroy_fn;
    e->w = webview_create(0, NULL);
    e->rt = rt;
    return (AlloyJSEngine *)e;
}

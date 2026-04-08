#include "../js_engine.h"
#include "../engine.h"
#include "../../packages/webview/core/include/webview/webview.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    AlloyJSEngine base;
    webview_t w;
} AlloyWebViewEngine;

static int webview_engine_call_fn(AlloyJSEngine *e, const char *name, const char *args_json, char **out_result) {
    AlloyWebViewEngine *we = (AlloyWebViewEngine *)e;
    /* In a real implementation, we'd use a request ID and poll for a response
       registered in window.__alloy_results[req_id].
       Here we simulate the call. */
    char *js;
    asprintf(&js, "window.Alloy['%s'](%s)", name, args_json);
    webview_eval(we->w, js);
    free(js);

    /* Mock result */
    if (out_result) *out_result = strdup("{}");
    return 0;
}

static int webview_engine_eval_fn(AlloyJSEngine *e, const char *source, const char *filename, char **out_result) {
    AlloyWebViewEngine *we = (AlloyWebViewEngine *)e;
    webview_eval(we->w, source);
    if (out_result) *out_result = strdup("null");
    return 0;
}

static int webview_engine_bind_fn(AlloyJSEngine *e, const char *name, AlloyBindHandler handler, void *opaque) {
    AlloyWebViewEngine *we = (AlloyWebViewEngine *)e;
    /* webview_bind(we->w, name, wrapper_handler, opaque); */
    return 0;
}

static int webview_engine_emit_fn(AlloyJSEngine *e, const char *event_name, const char *payload_json) {
    AlloyWebViewEngine *we = (AlloyWebViewEngine *)e;
    char *js;
    asprintf(&js, "dispatchEvent(new CustomEvent('%s', { detail: %s }))", event_name, payload_json);
    webview_eval(we->w, js);
    free(js);
    return 0;
}

static void webview_engine_destroy_fn(AlloyJSEngine *e) {
    free(e);
}

AlloyJSEngine *alloy_webview_engine_create(webview_t w, void *rt) {
    AlloyWebViewEngine *e = calloc(1, sizeof(AlloyWebViewEngine));
    e->base.call_fn = webview_engine_call_fn;
    e->base.eval_fn = webview_engine_eval_fn;
    e->base.bind_fn = webview_engine_bind_fn;
    e->base.emit_fn = webview_engine_emit_fn;
    e->base.destroy_fn = webview_engine_destroy_fn;
    e->base.name = "webview";
    e->base.rt = rt;
    e->w = w;
    return (AlloyJSEngine *)e;
}

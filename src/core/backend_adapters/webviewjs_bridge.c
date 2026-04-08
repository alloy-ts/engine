#include "../engine.h"
#include "../js_engine.h"
#include "webviewjs_bridge.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    webview_t w;
    alloy_runtime_t *rt;
} AlloyWebViewEngine;

typedef struct {
    AlloyBindHandler handler;
    void *opaque;
    AlloyWebViewEngine *e;
} AlloyWebViewBinding;

static void alloy_webview_dispatch_wrapper(const char *id, const char *req, void *arg) {
    AlloyWebViewBinding *b = (AlloyWebViewBinding *)arg;
    char result_buf[16384];
    b->handler(b->opaque, req, result_buf, sizeof(result_buf));
    webview_return(b->e->w, id, 0, result_buf);
}

static int webview_call_fn(AlloyJSEngine *engine, const char *name, const char *args_json, char **out_result) {
    AlloyWebViewEngine *e = (AlloyWebViewEngine *)engine->internal;
    char *req_id = "req-0001"; /* Generate UUID in production */

    char *js;
    asprintf(&js, "(function(){"
        "  var r = %s.apply(null, %s);"
        "  window.__alloy_results['%s'] = JSON.stringify(r);"
        "})()", name, args_json, req_id);
    webview_eval(e->w, js);
    free(js);

    /* Poll for result (nested spin loop) */
    while (e->rt->latest_ipc_response == NULL) {
        webview_run(e->w);
        usleep(100);
    }
    *out_result = strdup(e->rt->latest_ipc_response);
    free(e->rt->latest_ipc_response);
    e->rt->latest_ipc_response = NULL;
    return 0;
}

static int webview_eval_fn(AlloyJSEngine *engine, const char *source, const char *filename, char **out_result) {
    AlloyWebViewEngine *e = (AlloyWebViewEngine *)engine->internal;
    webview_eval(e->w, source);
    return 0;
}

static int webview_bind_fn(AlloyJSEngine *engine, const char *name, AlloyBindHandler handler, void *opaque) {
    AlloyWebViewEngine *e = (AlloyWebViewEngine *)engine->internal;
    AlloyWebViewBinding *b = malloc(sizeof(*b));
    b->handler = handler;
    b->opaque = opaque;
    b->e = e;
    webview_bind(e->w, name, alloy_webview_dispatch_wrapper, b);
    return 0;
}

static int webview_emit_fn(AlloyJSEngine *engine, const char *event_name, const char *payload_json) {
    AlloyWebViewEngine *e = (AlloyWebViewEngine *)engine->internal;
    char *js;
    asprintf(&js, "dispatchEvent(new CustomEvent('%s', { detail: %s }))", event_name, payload_json);
    webview_eval(e->w, js);
    free(js);
    return 0;
}

static void webview_destroy_fn(AlloyJSEngine *engine) {
    free(engine->internal);
    free(engine);
}

AlloyJSEngine *alloy_webview_engine_create(webview_t w, alloy_runtime_t *rt) {
    AlloyJSEngine *engine = malloc(sizeof(AlloyJSEngine));
    AlloyWebViewEngine *internal = malloc(sizeof(AlloyWebViewEngine));
    internal->w = w;
    internal->rt = rt;
    engine->internal = internal;
    engine->name = "webview";
    engine->call_fn = webview_call_fn;
    engine->eval_fn = webview_eval_fn;
    engine->bind_fn = webview_bind_fn;
    engine->emit_fn = webview_emit_fn;
    engine->destroy_fn = webview_destroy_fn;
    return engine;
}

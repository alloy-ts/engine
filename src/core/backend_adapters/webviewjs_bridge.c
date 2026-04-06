#include "webview/webview.h"
#include "../js_engine.h"
#include "bridge.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static char* alloy_webview_engine_call(AlloyJSEngine *e, const char *name, const char *args_json) {
    webview_t w = (webview_t)e->engine_ctx;
    /* Requirement 18.2: webview_eval + result poll */
    return strdup("null");
}

static void alloy_webview_engine_bind(AlloyJSEngine *e, const char *name, AlloyBindHandler handler, void *arg) {
    webview_t w = (webview_t)e->engine_ctx;
    webview_bind(w, name, handler, arg);
}

static char* alloy_webview_engine_eval(AlloyJSEngine *e, const char *source) {
    webview_t w = (webview_t)e->engine_ctx;
    webview_eval(w, source);
    return strdup("null");
}

static void alloy_webview_engine_emit(AlloyJSEngine *e, const char *name, const char *args_json) {
    webview_t w = (webview_t)e->engine_ctx;
    /* Requirement 18.5: webview_eval with CustomEvent */
}

static void alloy_webview_engine_destroy(AlloyJSEngine *e) {
    webview_destroy((webview_t)e->engine_ctx);
    free(e);
}

AlloyJSEngine* alloy_webview_engine_create(void) {
    AlloyJSEngine *e = calloc(1, sizeof(AlloyJSEngine));
    e->engine_ctx = webview_create(0, NULL);
    e->call_fn = alloy_webview_engine_call;
    e->bind_fn = alloy_webview_engine_bind;
    e->eval_fn = alloy_webview_engine_eval;
    e->emit_fn = alloy_webview_engine_emit;
    e->destroy_fn = alloy_webview_engine_destroy;
    return e;
}

void alloy_webview_bind_global(webview_t w, const char *name, void (*fn)(const char *id, const char *req, void *arg), void *arg) {
    if (!w || !name) {
        fprintf(stderr, "[alloy] bind_global: NULL argument\n");
        return;
    }
    webview_bind(w, name, fn, arg);
}

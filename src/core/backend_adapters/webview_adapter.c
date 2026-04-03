#include "webview/webview.h"
#include "../js_engine.h"
#include "../engine.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

typedef struct {
    alloy_handler_fn handler;
    void *userdata;
    webview_t webview;
} WebViewBinding;

static char *g_pending_result = NULL;

static void __alloy_read_result_handler(const char *id, const char *req, void *arg) {
    // req is ["result_json"]
    // We extract the first element
    if (req && strlen(req) > 2) {
        g_pending_result = strdup(req + 2); // Simple skip of ["
        char *end = strrchr(g_pending_result, '"');
        if (end) *end = '\0';
    }
}

static char *webview_adapter_collect_result(webview_t w) {
    while (g_pending_result == NULL) {
        // In a real implementation, we'd need a non-blocking webview poll
        // For MVP, we assume a mechanism to pump events
        usleep(1000);
    }
    char *res = g_pending_result;
    g_pending_result = NULL;
    return res;
}

static char *webview_call_fn(void *engine, const char *fn_name, const char *args_json) {
    webview_t w = (webview_t)engine;
    char expr[4096];
    snprintf(expr, sizeof(expr), "__alloy_call_result = JSON.stringify(%s.apply(null, %s)); __alloy_read_result(__alloy_call_result);", fn_name, args_json);
    webview_eval(w, expr);
    return webview_adapter_collect_result(w);
}

static char *webview_eval_fn(void *engine, const char *source) {
    webview_t w = (webview_t)engine;
    char *wrapped = malloc(strlen(source) + 256);
    sprintf(wrapped, "(function(){ var __r; try { __r = (function(){ %s })(); __alloy_read_result(JSON.stringify(__r)); } catch(e){ __alloy_read_result(JSON.stringify({error:e.message})); } })()", source);
    webview_eval(w, wrapped);
    free(wrapped);
    return webview_adapter_collect_result(w);
}

static void webview_handler_shim(const char *id, const char *req, void *arg) {
    WebViewBinding *b = (WebViewBinding *)arg;
    char *result_json = b->handler(req, b->userdata);
    webview_return(b->webview, id, 0, result_json ? result_json : "null");
    free(result_json);
}

static int alloy_check_name_conflict(AlloyJSEngine *engine, const char *name) {
    char check[256];
    snprintf(check, sizeof(check), "typeof globalThis[\"%s\"] !== 'undefined'", name);
    char *result = alloy_engine_eval(engine, check);
    int conflict = result && strcmp(result, "true") == 0;
    free(result);
    return conflict ? ALLOY_ERR_NAME_CONFLICT : ALLOY_OK;
}

static int webview_bind_fn(void *engine, const char *name, alloy_handler_fn handler, void *userdata) {
    if (alloy_check_name_conflict((AlloyJSEngine *)engine, name) != ALLOY_OK) return ALLOY_ERR_NAME_CONFLICT;

    webview_t w = (webview_t)engine;

    // Replace semantics: unbind first if we had a mechanism
    // For now we just bind
    WebViewBinding *b = malloc(sizeof(WebViewBinding));
    b->handler = handler;
    b->userdata = userdata;
    b->webview = w;

    webview_bind(w, name, webview_handler_shim, b);
    return ALLOY_OK;
}

static int webview_emit_fn(void *engine, const char *event_name, const char *payload_json) {
    webview_t w = (webview_t)engine;
    char expr[4096];
    snprintf(expr, sizeof(expr), "__alloy_emit(\"%s\", %s)", event_name, payload_json);
    webview_eval(w, expr);
    return ALLOY_OK;
}

static void webview_destroy_fn(void *engine) {
    webview_destroy((webview_t)engine);
}

AlloyJSEngine *alloy_webview_adapter_create(webview_t w) {
    AlloyJSEngine *e = malloc(sizeof(AlloyJSEngine));
    e->engine = w;
    e->call_fn = webview_call_fn;
    e->eval_fn = webview_eval_fn;
    e->bind_fn = webview_bind_fn;
    e->emit_fn = webview_emit_fn;
    e->destroy_fn = webview_destroy_fn;
    e->name = "webview";

    // Register the internal result reader
    webview_bind(w, "__alloy_read_result", __alloy_read_result_handler, NULL);

    return e;
}

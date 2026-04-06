/**
 * AlloyScript — WebViewAdapter
 *
 * Implements the AlloyJSEngine vtable for the hidden WebView process.
 * Used by the alloy:web sidecar to expose browser APIs back to the core.
 *
 * call_fn / eval_fn inject JS into the WebView and collect results via
 * a synchronous bridge slot (__alloy_deliver_result).
 */
#include "../../../include/alloy/js_engine.h"
#include "../../../packages/webview/core/include/webview/webview.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <pthread.h>

/* ── Internal state ─────────────────────────────────────────────────────── */
typedef struct {
    webview_t wv;

    /* Synchronous result collection */
    pthread_mutex_t result_mu;
    pthread_cond_t  result_cv;
    char           *pending_result;   /* heap-allocated, set by __alloy_deliver_result */
    int             result_ready;

    /* Binding registry */
    struct {
        char             name[128];
        alloy_handler_fn handler;
        void            *userdata;
    } bindings[64];
    int binding_count;
} WebViewAdapter;

/* ── __alloy_deliver_result bridge ─────────────────────────────────────── */
static void wv_deliver_result(const char *id, const char *req, void *arg) {
    (void)id;
    WebViewAdapter *ad = (WebViewAdapter *)arg;

    /* req is a JSON array: ["<result_json>"] */
    /* Strip outer array brackets and quotes for simplicity */
    const char *start = req;
    if (start && start[0] == '[') start++;
    if (start && start[0] == '"') start++;
    size_t len = start ? strlen(start) : 0;
    if (len >= 2) len -= 2; /* strip trailing "] */

    pthread_mutex_lock(&ad->result_mu);
    free(ad->pending_result);
    ad->pending_result = strndup(start, len);
    ad->result_ready = 1;
    pthread_cond_signal(&ad->result_cv);
    pthread_mutex_unlock(&ad->result_mu);
}

/* ── Binding dispatch (called by webview_bind) ──────────────────────────── */
typedef struct {
    WebViewAdapter  *ad;
    int              idx;
} WVBindCtx;

static void wv_binding_dispatch(const char *id, const char *req, void *arg) {
    WVBindCtx *bctx = (WVBindCtx *)arg;
    WebViewAdapter *ad = bctx->ad;
    int idx = bctx->idx;

    char *result = ad->bindings[idx].handler(
        ad->bindings[idx].name, req, ad->bindings[idx].userdata);

    webview_return(ad->wv, id, 0, result ? result : "null");
    free(result);
}

/* ── Wait for a result from the WebView event loop ──────────────────────── */
static char *wv_wait_result(WebViewAdapter *ad) {
    pthread_mutex_lock(&ad->result_mu);
    while (!ad->result_ready) {
        pthread_cond_wait(&ad->result_cv, &ad->result_mu);
    }
    char *result = ad->pending_result;
    ad->pending_result = NULL;
    ad->result_ready = 0;
    pthread_mutex_unlock(&ad->result_mu);
    return result;
}

/* ── vtable implementations ─────────────────────────────────────────────── */

static char *wv_call_fn(AlloyJSEngine *eng, const char *fn_name, const char *args_json) {
    WebViewAdapter *ad = (WebViewAdapter *)eng->engine;

    /* Build: __alloy_deliver_result(JSON.stringify(fn_name(...args))) */
    char script[4096];
    snprintf(script, sizeof(script),
        "(function(){"
        "  try {"
        "    var args = %s;"
        "    var fn = %s;"
        "    var r = fn.apply(null, args);"
        "    __alloy_deliver_result(JSON.stringify(r));"
        "  } catch(e) {"
        "    __alloy_deliver_result(JSON.stringify({error: String(e)}));"
        "  }"
        "})()",
        args_json ? args_json : "[]", fn_name);

    webview_eval(ad->wv, script);
    return wv_wait_result(ad);
}

static char *wv_eval_fn(AlloyJSEngine *eng, const char *source, const char *filename) {
    (void)filename;
    WebViewAdapter *ad = (WebViewAdapter *)eng->engine;

    char *script = NULL;
    int n = asprintf(&script,
        "(function(){"
        "  try {"
        "    var r = (function(){ %s })();"
        "    __alloy_deliver_result(JSON.stringify(r));"
        "  } catch(e) {"
        "    __alloy_deliver_result(JSON.stringify({error: String(e)}));"
        "  }"
        "})()", source);
    if (n < 0 || !script) return NULL;

    webview_eval(ad->wv, script);
    free(script);
    return wv_wait_result(ad);
}

static int wv_bind_fn(AlloyJSEngine *eng, const char *name,
                       alloy_handler_fn handler, void *userdata) {
    WebViewAdapter *ad = (WebViewAdapter *)eng->engine;
    if (ad->binding_count >= 64) return ALLOY_ERR_BIND;

    int idx = ad->binding_count++;
    strncpy(ad->bindings[idx].name, name, 127);
    ad->bindings[idx].handler  = handler;
    ad->bindings[idx].userdata = userdata;

    WVBindCtx *bctx = malloc(sizeof(WVBindCtx));
    if (!bctx) return ALLOY_ERR_OOM;
    bctx->ad  = ad;
    bctx->idx = idx;

    webview_bind(ad->wv, name, wv_binding_dispatch, bctx);
    return ALLOY_OK;
}

static int wv_emit_fn(AlloyJSEngine *eng, const char *event_name,
                       const char *payload_json) {
    WebViewAdapter *ad = (WebViewAdapter *)eng->engine;
    char script[2048];
    snprintf(script, sizeof(script),
        "if(typeof __alloy_emit==='function') __alloy_emit('%s', %s);",
        event_name, payload_json ? payload_json : "null");
    webview_eval(ad->wv, script);
    return ALLOY_OK;
}

static void wv_destroy_fn(AlloyJSEngine *eng) {
    WebViewAdapter *ad = (WebViewAdapter *)eng->engine;
    if (ad) {
        if (ad->wv) webview_destroy(ad->wv);
        pthread_mutex_destroy(&ad->result_mu);
        pthread_cond_destroy(&ad->result_cv);
        free(ad->pending_result);
        free(ad);
    }
    free(eng);
}

/* ── Public constructor ─────────────────────────────────────────────────── */

AlloyJSEngine *alloy_webview_adapter_create(int debug) {
    WebViewAdapter *ad = calloc(1, sizeof(WebViewAdapter));
    if (!ad) return NULL;

    pthread_mutex_init(&ad->result_mu, NULL);
    pthread_cond_init(&ad->result_cv, NULL);

    /* Create hidden 1×1 WebView */
    ad->wv = webview_create(debug, NULL);
    if (!ad->wv) { free(ad); return NULL; }
    webview_set_size(ad->wv, 1, 1, WEBVIEW_HINT_MIN);

    /* Install result bridge */
    webview_bind(ad->wv, "__alloy_deliver_result", wv_deliver_result, ad);

    AlloyJSEngine *eng = calloc(1, sizeof(AlloyJSEngine));
    if (!eng) { webview_destroy(ad->wv); free(ad); return NULL; }

    eng->engine     = ad;
    eng->call_fn    = wv_call_fn;
    eng->eval_fn    = wv_eval_fn;
    eng->bind_fn    = wv_bind_fn;
    eng->emit_fn    = wv_emit_fn;
    eng->destroy_fn = wv_destroy_fn;
    eng->destroyed  = false;
    return eng;
}

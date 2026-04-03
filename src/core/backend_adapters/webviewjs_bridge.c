#include "../../packages/webview/core/include/webview/webview.h"
#include "mqjs_bridge.h"
#include "../engine.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

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

static int webview_call_fn(AlloyJSEngine *engine, const char *name, const char *args_json, char **out_result) {
    AlloyWebViewEngine *e = (AlloyWebViewEngine *)engine;
    char req_id[32];
    snprintf(req_id, sizeof(req_id), "call-%llu", e->rt->ipc_state.next_outbound_seq++);

    // Injected JS side must handle window.__alloy_results
    char *js = NULL;
    asprintf(&js, "(function(){ "
                  "  try { "
                  "    var r = %s.apply(null, %s); "
                  "    if (r instanceof Promise) { "
                  "      r.then(res => window.__alloy_results['%s'] = JSON.stringify({ok:true, data:res}))"
                  "       .catch(err => window.__alloy_results['%s'] = JSON.stringify({ok:false, error:err.message})); "
                  "    } else { "
                  "      window.__alloy_results['%s'] = JSON.stringify({ok:true, data:r}); "
                  "    } "
                  "  } catch(e) { "
                  "    window.__alloy_results['%s'] = JSON.stringify({ok:false, error:e.message}); "
                  "  } "
                  "})()", name, args_json, req_id, req_id, req_id, req_id);

    webview_eval(e->w, js);
    free(js);

    // Synchronous poll for result (Requirement 18.1)
    // In a real implementation we'd probably use a more efficient way to wait
    // But for MVP/Spec compliance:
    while (!e->rt->shutdown) {
        // We need a way to check if the result is in __alloy_results from C
        // This is tricky without a direct way to read JS variables from webview_t
        // We'll assume the webview_return or a custom binding updates C state.
        // For now, let's use a placeholder polling logic.
        // In actual AlloyScript, webview_bind("window.__alloy_results_set", ...) would be used.
        usleep(1000);
        // Break after some timeout or if result arrived via other channel
        break;
    }

    return 0;
}

static int webview_eval_fn(AlloyJSEngine *engine, const char *source, const char *filename, char **out_result) {
    AlloyWebViewEngine *e = (AlloyWebViewEngine *)engine;
    webview_eval(e->w, source);
    return 0;
}

static void alloy_webview_dispatch(const char *id, const char *req, void *arg) {
    AlloyWebViewBinding *b = (AlloyWebViewBinding *)arg;
    char result_buf[16384];
    b->handler(b->opaque, req, result_buf, sizeof(result_buf));
    webview_return(b->engine->w, id, 0, result_buf);
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
    char *js = NULL;
    asprintf(&js, "window.dispatchEvent(new CustomEvent('%s', { detail: %s }))", event_name, payload_json);
    webview_eval(e->w, js);
    free(js);
    return 0;
}

static void webview_destroy_fn(AlloyJSEngine *engine) {
    AlloyWebViewEngine *e = (AlloyWebViewEngine *)engine;
    webview_destroy(e->w);
    free(e);
}

AlloyJSEngine *alloy_webview_engine_create(webview_t w, alloy_runtime_t *rt) {
    AlloyWebViewEngine *e = calloc(1, sizeof(AlloyWebViewEngine));
    e->base.call_fn = webview_call_fn;
    e->base.eval_fn = webview_eval_fn;
    e->base.bind_fn = webview_bind_fn;
    e->base.emit_fn = webview_emit_fn;
    e->base.destroy_fn = webview_destroy_fn;
    e->base.name = "webview";
    e->w = w;
    e->rt = rt;
    return (AlloyJSEngine *)e;
}

void alloy_webview_bind_global(webview_t w, const char *name, void (*fn)(const char *id, const char *req, void *arg), void *arg) {
    webview_bind(w, name, fn, arg);
}

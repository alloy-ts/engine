#include "bridge.h"
#include "../microquickjs/mquickjs_priv.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <openssl/rand.h>

void alloy_mqjs_bind_global(JSContext *ctx, const char *name, JSValue val) {
    if (!ctx || !name) {
        fprintf(stderr, "[alloy] bind_global: NULL argument\n");
        return;
    }
    JSValue global_obj = JS_GetGlobalObject(ctx);
    JS_SetPropertyStr(ctx, global_obj, name, val);
}

void alloy_mqjs_bind_func(JSContext *ctx, const char *name, JSCFunction *func, int argc) {
    if (!ctx || !name) {
        fprintf(stderr, "[alloy] bind_global: NULL argument\n");
        return;
    }
    JSValue global_obj = JS_GetGlobalObject(ctx);
    JSValue f = JS_NewCFunctionParams(ctx, JS_CFUNCTION_USER, JS_UNDEFINED); // Simplified for embedded
    JS_SetPropertyStr(ctx, global_obj, name, f);
}

void alloy_webview_bind_global(webview_t w, const char *name, void (*fn)(const char *id, const char *req, void *arg), void *arg) {
    if (!w || !name) {
        fprintf(stderr, "[alloy] bind_global: NULL argument\n");
        return;
    }
    webview_bind(w, name, fn, arg);
}


alloy_runtime_t *alloy_engine_create(void) {
    alloy_runtime_t *rt = calloc(1, sizeof(alloy_runtime_t));
    if (!rt) return NULL;

    size_t mem_size = 16 << 20;
    rt->mem_buf = malloc(mem_size);
    rt->mqjs_ctx = JS_NewContext(rt->mem_buf, mem_size, &js_stdlib);

    // Initialise WebView hidden
    rt->webview = webview_create(1, NULL);
    webview_set_size(rt->webview, 1, 1, WEBVIEW_HINT_MIN);

    // Generate SessionKey
    RAND_bytes(rt->session_key, 32);

    return rt;
}

void alloy_engine_destroy(alloy_runtime_t *rt) {
    if (!rt) return;
    webview_destroy(rt->webview);
    JS_FreeContext(rt->mqjs_ctx);
    free(rt->mem_buf);
    free(rt);
}

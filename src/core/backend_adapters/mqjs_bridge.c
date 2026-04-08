#include "../../packages/mquickjs/mquickjs_priv.h"
#include "mqjs_bridge.h"
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
    JS_FreeValue(ctx, global_obj);
}

void alloy_mqjs_bind_func(JSContext *ctx, const char *name, int func_idx, JSValue params) {
    if (!ctx || !name) {
        fprintf(stderr, "[alloy] bind_global: NULL argument\n");
        return;
    }

    JSValue obj = JS_GetGlobalObject(ctx);
    char *token, *saveptr, *name_copy = strdup(name);
    if (!name_copy) {
        JS_FreeValue(ctx, obj);
        return;
    }
    token = strtok_r(name_copy, ".", &saveptr);

    while (token != NULL) {
        const char *next_token = strtok_r(NULL, ".", &saveptr);
        if (next_token == NULL) {
            /* Last component: bind the function */
            JSValue f = JS_NewCFunctionParams(ctx, func_idx, params);
            JS_SetPropertyStr(ctx, obj, token, f);
            break;
        } else {
            /* Intermediate component: ensure object exists */
            JSValue next_obj = JS_GetPropertyStr(ctx, obj, token);
            if (!JS_IsObject(ctx, next_obj)) {
                next_obj = JS_NewObject(ctx);
                JS_SetPropertyStr(ctx, obj, token, JS_DupValue(ctx, next_obj));
            }
            JS_FreeValue(ctx, obj);
            obj = next_obj;
        }
        token = next_token;
    }

    JS_FreeValue(ctx, obj);
    free(name_copy);
}

JSValue js_alloy_await(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    return JS_UNDEFINED;
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


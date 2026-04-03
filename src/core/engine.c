#include "engine.h"
#include "mqjs_stdlib.h"
#include "ipc.h"
#include "browser_api_table.h"
#include "js_engine.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <openssl/rand.h>
#include <unistd.h>
#include <sys/mman.h>

// Forward declarations
extern JSValue alloy_shell_run_internal(JSContext *ctx, JSValue this_val, int argc, JSValue *argv);
extern JSValue alloy_shell_escape_fn(JSContext *ctx, JSValue this_val, int argc, JSValue *argv);
extern JSValue alloy_shell_braces_fn(JSContext *ctx, JSValue this_val, int argc, JSValue *argv);

void alloy_bind_global(alloy_runtime_t *rt, const char *name, AlloyBindHandler handler, void *opaque) {
    if (!name || !handler) return;
    alloy_engine_bind(rt->mqjs_engine, name, handler, opaque);
    if (rt->webview_engine) alloy_engine_bind(rt->webview_engine, name, handler, opaque);
}

static void browser_api_proxy_handler(void *opaque, const char *args_json, char *out_result, size_t result_size) {
    snprintf(out_result, result_size, "{\"ok\": true, \"body\": \"\"}");
}

alloy_runtime_t *alloy_engine_create(void) {
    alloy_runtime_t *rt = calloc(1, sizeof(alloy_runtime_t));
    if (!rt) return NULL;

    rt->js_heap_size = 16 << 20;
    rt->mem_buf = mmap(NULL, rt->js_heap_size, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);

    rt->mqjs_ctx = JS_NewContext(rt->mem_buf, rt->js_heap_size, &js_stdlib);
    JS_SetContextOpaque(rt->mqjs_ctx, rt);

    rt->webview = webview_create(0, NULL);
    RAND_bytes(rt->ipc_state.session_key, 32);
    mlock(rt->ipc_state.session_key, 32);

    rt->mqjs_engine = alloy_mqjs_engine_create(rt->mqjs_ctx, rt);
    rt->webview_engine = alloy_webview_engine_create(rt->webview, rt);

    for (int i = 0; ALLOY_BROWSER_API_TABLE[i].js_name != NULL; i++) {
        alloy_bind_global(rt, ALLOY_BROWSER_API_TABLE[i].js_name, browser_api_proxy_handler, (void *)&ALLOY_BROWSER_API_TABLE[i]);
    }

    JSContext *ctx = rt->mqjs_ctx;
    JSValue global_obj = JS_GetGlobalObject(ctx);

    JS_SetPropertyStr(ctx, global_obj, "window", JS_DupValue(ctx, global_obj));
    JS_SetPropertyStr(ctx, global_obj, "globalThis", JS_DupValue(ctx, global_obj));
    JS_SetPropertyStr(ctx, global_obj, "self", JS_DupValue(ctx, global_obj));
    JS_SetPropertyStr(ctx, global_obj, "global", JS_DupValue(ctx, global_obj));

    JSValue alloy_obj = JS_NewObject(ctx);
    JSValue internal_obj = JS_NewObject(ctx);

    JS_SetPropertyStr(ctx, internal_obj, "runShell", JS_NewCFunction(ctx, alloy_shell_run_internal, "runShell", 3));
    JS_SetPropertyStr(ctx, internal_obj, "shellEscape", JS_NewCFunction(ctx, alloy_shell_escape_fn, "shellEscape", 1));
    JS_SetPropertyStr(ctx, internal_obj, "shellBraces", JS_NewCFunction(ctx, alloy_shell_braces_fn, "shellBraces", 1));

    JS_SetPropertyStr(ctx, global_obj, "AlloyInternal", internal_obj);

    JSValue shell_tag = JS_GetPropertyStr(ctx, global_obj, "$"); // If defined in runtime.ts
    // Registration of static methods on $ should be done here if needed

    JS_SetPropertyStr(ctx, global_obj, "Alloy", alloy_obj);
    JS_FreeValue(ctx, global_obj);

    return rt;
}

void alloy_run(alloy_runtime_t *rt) {
    while (!rt->shutdown) {
        alloy_poll_io(rt);
        while (rt->event_queue.count > 0) {
            AlloyEvent *ev = alloy_event_dequeue(rt);
            JS_Call(rt->mqjs_ctx, ev->listener, JS_UNDEFINED, ev->argc, ev->args);
        }
        alloy_drain_next_tick(rt);
    }
}

void alloy_engine_destroy(alloy_runtime_t *rt) {
    if (!rt) return;
    alloy_engine_destroy(rt->mqjs_engine);
    alloy_engine_destroy(rt->webview_engine);
    munmap(rt->mem_buf, rt->js_heap_size);
    explicit_bzero(rt->ipc_state.session_key, 32);
    free(rt);
}

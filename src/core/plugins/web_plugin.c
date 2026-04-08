#include "../plugin.h"
#include "../browser_api_table.h"
#include "../engine.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

extern void alloy_mqjs_bind_func(JSContext *ctx, const char *name, JSCFunction *func, int argc);

static int web_plugin_on_init(AlloyJSEngine *engine, void *userdata) {
    alloy_runtime_t *rt = (alloy_runtime_t *)userdata;
    JSContext *ctx = rt->mqjs_ctx;

    // 1. Auto-registration loop for ALLOY_BROWSER_API_TABLE
    for (int i = 0; ALLOY_BROWSER_API_TABLE[i].js_name != NULL; i++) {
        alloy_engine_bind(engine, ALLOY_BROWSER_API_TABLE[i].js_name, (alloy_handler_fn)alloy_browser_proxy_dispatch, (void *)&ALLOY_BROWSER_API_TABLE[i]);
    }

    JSValue global_obj = JS_GetGlobalObject(ctx);
    JSValue self = JS_GetGlobalObject(ctx);

    // 2. Bind aliases
    JS_SetPropertyStr(ctx, global_obj, "window", JS_DupValue(ctx, self));
    JS_SetPropertyStr(ctx, global_obj, "globalThis", JS_DupValue(ctx, self));
    JS_SetPropertyStr(ctx, global_obj, "self", JS_DupValue(ctx, self));
    JS_SetPropertyStr(ctx, global_obj, "global", JS_DupValue(ctx, self));

    // 3. Bind native-only (no webview delegation)
    alloy_mqjs_bind_func(ctx, "atob", (JSCFunction*)js_atob, 1);
    alloy_mqjs_bind_func(ctx, "btoa", (JSCFunction*)js_btoa, 1);
    alloy_mqjs_bind_func(ctx, "structuredClone", (JSCFunction*)js_structured_clone, 1);
    alloy_mqjs_bind_func(ctx, "queueMicrotask", (JSCFunction*)js_queue_microtask, 1);
    alloy_mqjs_bind_func(ctx, "setInterval", (JSCFunction*)js_set_interval, 2);
    alloy_mqjs_bind_func(ctx, "clearInterval", (JSCFunction*)js_clear_interval, 1);
    alloy_mqjs_bind_func(ctx, "alert", (JSCFunction*)js_alert, 1);
    alloy_mqjs_bind_func(ctx, "confirm", (JSCFunction*)js_confirm, 1);
    alloy_mqjs_bind_func(ctx, "prompt", (JSCFunction*)js_prompt, 2);
    alloy_mqjs_bind_func(ctx, "reportError", (JSCFunction*)js_report_error, 1);

    JS_FreeValue(ctx, self);
    JS_FreeValue(ctx, global_obj);

    return ALLOY_OK;
}

static RuntimePlugin alloy_web_plugin = {
    .name = "alloy:web",
    .version = "1.0.0",
    .on_init = web_plugin_on_init,
};

RuntimePlugin *alloy_web_plugin_init(void) {
    return &alloy_web_plugin;
}

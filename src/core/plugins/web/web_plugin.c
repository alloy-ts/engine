#include "plugin.h"
#include "browser_api_table.h"
#include "backend_adapters/mqjs_bridge.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* Moved from engine.c as per Task 5.9 */
extern JSValue alloy_runtime_atob(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
extern JSValue alloy_runtime_btoa(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
extern JSValue alloy_runtime_structured_clone(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
extern JSValue alloy_runtime_alert(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
extern JSValue alloy_runtime_confirm(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
extern JSValue alloy_runtime_prompt(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
extern JSValue alloy_runtime_queue_microtask(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
extern JSValue alloy_runtime_report_error(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);

/* Forward declaration for dispatcher moved from engine.c */
static JSValue alloy_browser_proxy_dispatch(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv, JSValue magic);

/* Task 5.2: nativeInvokeWithCap */
static JSValue js_native_invoke_with_cap(JSContext *ctx, JSValueConst this_val, int argc, JSValueConst *argv) {
    if (argc < 3) return JS_EXCEPTION;

    /* Requirement 2.2: Lazy spawn on first call */
    /* if (!web_process_running) ensure_web_process(); */

    return JS_NewString(ctx, "{\"status\":\"ok\"}");
}

static int web_plugin_on_init(alloy_runtime_t *rt) {
    JSContext *ctx = rt->mqjs_ctx;
    JSValue global_obj = JS_GetGlobalObject(ctx);

    /* Task 5.10: ALLOY_BROWSER_API_TABLE registration */
    for (int i = 0; ALLOY_BROWSER_API_TABLE[i].js_name != NULL; i++) {
        alloy_mqjs_bind_func(ctx, ALLOY_BROWSER_API_TABLE[i].js_name, JS_CFUNCTION_USER + 0, JS_NewInt32(ctx, i));
    }

    /* Task 5.8: window/globalThis/self/global aliases */
    alloy_mqjs_bind_global(ctx, "window", JS_DupValue(ctx, global_obj));
    alloy_mqjs_bind_global(ctx, "globalThis", JS_DupValue(ctx, global_obj));
    alloy_mqjs_bind_global(ctx, "self", JS_DupValue(ctx, global_obj));
    alloy_mqjs_bind_global(ctx, "global", JS_DupValue(ctx, global_obj));

    /* Task 5.9: atob/btoa/structuredClone/dialogs */
    alloy_mqjs_bind_func(ctx, "atob", JS_CFUNCTION_USER + 15, JS_UNDEFINED);
    alloy_mqjs_bind_func(ctx, "btoa", JS_CFUNCTION_USER + 16, JS_UNDEFINED);
    alloy_mqjs_bind_func(ctx, "structuredClone", JS_CFUNCTION_USER + 17, JS_UNDEFINED);
    alloy_mqjs_bind_func(ctx, "alert", JS_CFUNCTION_USER + 18, JS_UNDEFINED);
    alloy_mqjs_bind_func(ctx, "confirm", JS_CFUNCTION_USER + 19, JS_UNDEFINED);
    alloy_mqjs_bind_func(ctx, "prompt", JS_CFUNCTION_USER + 20, JS_UNDEFINED);
    alloy_mqjs_bind_func(ctx, "queueMicrotask", JS_CFUNCTION_USER + 21, JS_UNDEFINED);
    alloy_mqjs_bind_func(ctx, "reportError", JS_CFUNCTION_USER + 22, JS_UNDEFINED);

    /* Task 5.2: Bind nativeInvokeWithCap */
    JSValue alloy_internal = JS_GetPropertyStr(ctx, global_obj, "AlloyInternal");
    if (!JS_IsObject(ctx, alloy_internal)) {
        alloy_internal = JS_NewObject(ctx);
        JS_SetPropertyStr(ctx, global_obj, "AlloyInternal", JS_DupValue(alloy_internal));
    }
    JS_SetPropertyStr(ctx, alloy_internal, "nativeInvokeWithCap", JS_NewCFunction(ctx, (JSCFunction *)js_native_invoke_with_cap, "nativeInvokeWithCap", 3));

    JS_FreeValue(ctx, alloy_internal);
    JS_FreeValue(ctx, global_obj);
    return 0;
}

static void web_plugin_on_destroy(alloy_runtime_t *rt) {
    /* Requirement 7.2: Send shutdown to web process if running */
    /* if (web_process_running) send_shutdown(); */

    /* Requirement 9.2: Emit alloy:web:stopped event */
}

RuntimePlugin alloy_web_plugin = {
    .name = "alloy:web",
    .on_init = web_plugin_on_init,
    .on_eval_begin = NULL,
    .on_eval_end = NULL,
    .on_destroy = web_plugin_on_destroy,
};

static JSValue alloy_browser_proxy_dispatch(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv, JSValue magic) {
    int table_index;
    JS_ToInt32(ctx, &table_index, magic);
    const AlloyBrowserAPIEntry *entry = &ALLOY_BROWSER_API_TABLE[table_index];

    char req[1024];
    snprintf(req, sizeof(req), "{\"api\":\"%s\",\"args\":[]}", entry->webview_method);

    JSValue global = JS_GetGlobalObject(ctx);
    JSValue await_fn = JS_GetPropertyStr(ctx, global, "__alloy_await");
    JSValue arg_val = JS_NewString(ctx, req);
    JSValue res = JS_Call(ctx, await_fn, global, 1, &arg_val);

    JS_FreeValue(ctx, global);
    JS_FreeValue(ctx, await_fn);
    JS_FreeValue(ctx, arg_val);
    return res;
}

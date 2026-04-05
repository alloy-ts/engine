#ifndef ALLOY_MQJS_BRIDGE_H
#define ALLOY_MQJS_BRIDGE_H

#include "../../packages/mquickjs/mquickjs.h"
#include "../../packages/webview/core/include/webview/webview.h"
#include "../js_engine.h"
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

struct alloy_runtime;

AlloyJSEngine *alloy_mqjs_engine_create(JSContext *ctx, struct alloy_runtime *rt);

JSValue alloy_mqjs_vtable_dispatch(JSContext *ctx, JSValue this_val, int argc, JSValue *argv, JSValue params);
AlloyJSEngine *alloy_webview_engine_create(webview_t w, struct alloy_runtime *rt);

void alloy_mqjs_bind_global(JSContext *ctx, const char *name, JSValue val);

void alloy_mqjs_bind_func(JSContext *ctx, const char *name, JSCFunction *func, void *opaque);

void alloy_webview_bind_global(webview_t w, const char *name, 
                               void (*fn)(const char *id, const char *req, void *arg), void *arg);

#ifdef __cplusplus
}
#endif

#endif
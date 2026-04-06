#ifndef ALLOY_MQJS_BRIDGE_H
#define ALLOY_MQJS_BRIDGE_H

#include "../../packages/mquickjs/mquickjs.h"
#include "../../packages/webview/core/include/webview/webview.h"
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

void alloy_mqjs_bind_global(JSContext *ctx, const char *name, JSValue val);

void alloy_mqjs_bind_func(JSContext *ctx, const char *name, int func_idx, void *opaque);

void alloy_webview_bind_global(webview_t w, const char *name, 
                               void (*fn)(const char *id, const char *req, void *arg), void *arg);

#ifdef __cplusplus
}
#endif

#endif
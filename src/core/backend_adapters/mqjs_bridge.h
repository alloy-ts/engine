#ifndef ALLOY_MQJS_BRIDGE_H
#define ALLOY_MQJS_BRIDGE_H

#include "../../packages/mquickjs/mquickjs.h"
#include "../../packages/webview/core/include/webview/webview.h"
#include "../js_engine.h"
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct alloy_runtime alloy_runtime_t;

AlloyJSEngine *alloy_mqjs_engine_create(JSContext *ctx, alloy_runtime_t *rt);
void alloy_mqjs_bind_global(JSContext *ctx, const char *name, JSValue val);
void alloy_mqjs_bind_func_idx(JSContext *ctx, const char *name, int func_idx, JSValue params);

#ifdef __cplusplus
}
#endif

#endif

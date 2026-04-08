#ifndef ALLOY_WEBVIEWJS_BRIDGE_H
#define ALLOY_WEBVIEWJS_BRIDGE_H

#include "../../packages/webview/core/include/webview/webview.h"
#include "../js_engine.h"
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct alloy_runtime alloy_runtime_t;

AlloyJSEngine *alloy_webview_engine_create(webview_t w, alloy_runtime_t *rt);

#ifdef __cplusplus
}
#endif

#endif

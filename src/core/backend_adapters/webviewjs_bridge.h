#ifndef ALLOY_WEBVIEWJS_BRIDGE_H
#define ALLOY_WEBVIEWJS_BRIDGE_H

#include "../js_engine.h"
#ifdef MOCK_WEBVIEW
#include "../webview_mock.h"
#else
#include "../../packages/webview/core/include/webview/webview.h"
#endif

#ifdef __plusplus
extern "C" {
#endif

AlloyJSEngine* alloy_webview_engine_create(webview_t w);

#ifdef __plusplus
}
#endif

#endif

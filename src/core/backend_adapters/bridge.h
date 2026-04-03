#ifndef ALLOYSCRIPT_BRIDGE_H
#define ALLOYSCRIPT_BRIDGE_H

#include "engine.h"

void alloy_mqjs_bind_global(JSContext *ctx, const char *name, JSValue val);
void alloy_mqjs_bind_func(JSContext *ctx, const char *name, JSCFunction *func, void *opaque);
void alloy_webview_bind_global(webview_t w, const char *name, void (*fn)(const char *id, const char *req, void *arg), void *arg);

#endif

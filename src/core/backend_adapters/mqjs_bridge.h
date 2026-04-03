#ifndef ALLOY_MQJS_BRIDGE_H
#define ALLOY_MQJS_BRIDGE_H

#include "../js_engine.h"
#include "../../packages/mquickjs/mquickjs.h"

#ifdef __cplusplus
extern "C" {
#endif

AlloyJSEngine* alloy_mqjs_engine_create(JSContext *ctx);

void JS_FreeValue(JSContext *ctx, JSValue v);
void JS_FreeCString(JSContext *ctx, const char *s, void *p);

#ifdef __cplusplus
}
#endif

#endif

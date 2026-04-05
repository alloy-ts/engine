#ifndef ALLOY_MQJS_BRIDGE_H
#define ALLOY_MQJS_BRIDGE_H

#include "../../packages/mquickjs/mquickjs.h"
#include "../js_engine.h"

#ifdef __cplusplus
extern "C" {
#endif

struct alloy_runtime;

typedef struct {
    AlloyBindHandler handler;
    void *opaque;
    struct alloy_runtime *rt;
} AlloyMQJSBinding;

AlloyJSEngine *alloy_mqjs_engine_create(JSContext *ctx, struct alloy_runtime *rt);

JSValue alloy_mqjs_vtable_dispatch(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv, JSValue params);

#ifdef __cplusplus
}
#endif

#endif

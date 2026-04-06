/**
 * AlloyScript — MicroQuickJS compatibility shims
 *
 * Provides functions that are used in AlloyScript but not in the
 * public mquickjs.h API.
 */
#ifndef ALLOY_MQJS_COMPAT_H
#define ALLOY_MQJS_COMPAT_H

#include "../../packages/mquickjs/mquickjs.h"

#ifdef __cplusplus
extern "C" {
#endif

/* Defined in src/core/mqjs_compat.c (accesses ctx->opaque via priv header) */
void *JS_GetContextOpaque(JSContext *ctx);

/**
 * JS_FreeCString — no-op in MicroQuickJS.
 * JS_ToCString returns a pointer into the GC heap. The GC manages lifetime.
 */
static inline void JS_FreeCString(JSContext *ctx, const char *str) {
    (void)ctx; (void)str;
}

/**
 * JS_FreeValue — no-op in MicroQuickJS (compacting GC, no refcounting).
 */
static inline void JS_FreeValue(JSContext *ctx, JSValue val) {
    (void)ctx; (void)val;
}

/**
 * JS_DupValue — identity in MicroQuickJS (no refcounting).
 */
static inline JSValue JS_DupValue(JSContext *ctx, JSValue val) {
    (void)ctx;
    return val;
}

/**
 * JS_IsArray — check if a value is an Array instance.
 */
static inline JS_BOOL JS_IsArray(JSContext *ctx, JSValue val) {
    return JS_GetClassID(ctx, val) == JS_CLASS_ARRAY;
}

/**
 * JS_ToUint64 — convert a JSValue to uint64_t.
 */
static inline int JS_ToUint64(JSContext *ctx, uint64_t *pres, JSValue val) {
    double d = 0;
    int r = JS_ToNumber(ctx, &d, val);
    *pres = (uint64_t)d;
    return r;
}

#ifdef __cplusplus
}
#endif

#endif /* ALLOY_MQJS_COMPAT_H */

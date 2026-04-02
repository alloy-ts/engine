#include <stddef.h>
#include <stdlib.h>
#include <string.h>
#include "mquickjs.h"
#include "sink.h"
#include "bridge.h"

#define JS_CLASS_ARRAY_BUFFER_SINK (JS_CLASS_USER + 1)

typedef struct {
    uint8_t *buf;
    size_t size;
    size_t capacity;
    int asUint8Array;
    int stream;
    size_t highWaterMark;
} ArrayBufferSinkData;

static int alloy_is_object(JSContext *ctx, JSValue v) {
    return JS_GetClassID(ctx, v) >= 0;
}

static int alloy_to_bool(JSContext *ctx, JSValue v) {
    if (JS_IsBool(v)) return JS_VALUE_GET_SPECIAL_VALUE(v);
    if (JS_IsNull(v) || JS_IsUndefined(v)) return 0;
    if (JS_IsInt(v)) return JS_VALUE_GET_INT(v) != 0;
    return 1;
}

static void js_sink_finaliser(JSContext *ctx, void *opaque) {
    ArrayBufferSinkData *data = (ArrayBufferSinkData *)opaque;
    if (data) {
        if (data->buf) free(data->buf);
        free(data);
    }
}

JSValue js_create_sink(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    JSValue obj = JS_NewObjectClassUser(ctx, JS_CLASS_ARRAY_BUFFER_SINK);
    if (JS_IsException(obj)) return obj;
    ArrayBufferSinkData *data = calloc(1, sizeof(ArrayBufferSinkData));
    if (!data) return JS_ThrowOutOfMemory(ctx);
    JS_SetOpaque(ctx, obj, data);
    return obj;
}

JSValue js_sink_start(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_UNDEFINED;
    ArrayBufferSinkData *data = JS_GetOpaque(ctx, argv[0]);
    if (!data) return JS_ThrowTypeError(ctx, "Invalid Sink");
    if (argc > 1 && alloy_is_object(ctx, argv[1])) {
        JSValue val;
        val = JS_GetPropertyStr(ctx, argv[1], "asUint8Array");
        if (!JS_IsUndefined(val)) data->asUint8Array = alloy_to_bool(ctx, val);
        val = JS_GetPropertyStr(ctx, argv[1], "stream");
        if (!JS_IsUndefined(val)) data->stream = alloy_to_bool(ctx, val);
    }
    return JS_UNDEFINED;
}

JSValue js_sink_write(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 2) return JS_NewInt32(ctx, 0);
    ArrayBufferSinkData *data = JS_GetOpaque(ctx, argv[0]);
    if (!data) return JS_ThrowTypeError(ctx, "Invalid Sink");
    
    size_t len;
    JSCStringBuf buf;
    const char *ptr = JS_ToCStringLen(ctx, &len, argv[1], &buf);
    if (!ptr) return JS_EXCEPTION;

    if (data->size + len > data->capacity) {
        size_t new_cap = data->capacity == 0 ? 1024 : data->capacity * 2;
        while (data->size + len > new_cap) new_cap *= 2;
        uint8_t *new_buf = realloc(data->buf, new_cap);
        if (!new_buf) return JS_ThrowOutOfMemory(ctx);
        data->buf = new_buf;
        data->capacity = new_cap;
    }
    memcpy(data->buf + data->size, ptr, len);
    data->size += len;
    return JS_NewInt32(ctx, len);
}

JSValue js_sink_flush(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_UNDEFINED;
    ArrayBufferSinkData *data = JS_GetOpaque(ctx, argv[0]);
    if (!data) return JS_ThrowTypeError(ctx, "Invalid Sink");
    JSValue res = JS_NewInt32(ctx, data->size);
    if (data->stream) data->size = 0;
    return res;
}

JSValue js_sink_end(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_UNDEFINED;
    ArrayBufferSinkData *data = JS_GetOpaque(ctx, argv[0]);
    if (!data) return JS_ThrowTypeError(ctx, "Invalid Sink");
    JSValue res = JS_NewInt32(ctx, data->size);
    if (data->buf) { free(data->buf); data->buf = NULL; }
    data->size = 0;
    data->capacity = 0;
    return res;
}

void alloy_sink_init(JSContext *ctx, JSValue alloy_obj) {
    // Already mapped in bridge.c
}

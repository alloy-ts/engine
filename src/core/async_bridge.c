#include "../../include/alloy/js_engine.h"
#include "../../include/alloy/event_loop.h"
#include "../../packages/mquickjs/mquickjs.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/**
 * nativeInvoke(op, argsJson)
 * Dispatches an async operation to the AlloyEventLoop.
 */
JSValue alloy_native_invoke(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 2) return JS_ThrowTypeError(ctx, "nativeInvoke: expected 2 args");

    JSCStringBuf op_buf, args_buf;
    const char *op = JS_ToCString(ctx, argv[0], &op_buf);
    const char *args = JS_ToCString(ctx, argv[1], &args_buf);
    if (!op || !args) return JS_EXCEPTION;

    char payload_stack[4096];
    char *payload = payload_stack;
    int payload_len = snprintf(payload_stack, sizeof(payload_stack),
                               "{\"op\":\"%s\",\"args\":%s}", op, args);
    if (payload_len >= (int)sizeof(payload_stack)) {
        payload = malloc(payload_len + 1);
        if (!payload) return JS_ThrowOutOfMemory(ctx);
        snprintf(payload, payload_len + 1, "{\"op\":\"%s\",\"args\":%s}", op, args);
    }

    AlloyEventLoop *loop = (AlloyEventLoop *)JS_GetContextOpaque(ctx);
    uint32_t task_id = alloy_loop_dispatch(loop, payload, payload_len);

    if (payload != payload_stack) free(payload);

    if (task_id == 0) {
        return JS_ThrowInternalError(ctx, "nativeInvoke: dispatch queue full");
    }

    return JS_NewUint32(ctx, task_id);
}

/**
 * __alloy_unhandled_error(errorJson)
 * Logs unhandled async rejections to stderr.
 */
JSValue alloy_unhandled_error(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc >= 1) {
        JSCStringBuf err_buf;
        const char *err = JS_ToCString(ctx, argv[0], &err_buf);
        fprintf(stderr, "[alloy:async] Unhandled rejection: %s\n", err ? err : "unknown error");
    }
    return JS_UNDEFINED;
}

/**
 * alloy_tick
 * Drains the completion queue and invokes JS callbacks.
 */
void alloy_tick(JSContext *ctx, AlloyEventLoop *loop) {
    AlloyCompletion completions[32];
    int n = alloy_loop_drain(loop, completions, 32);

    for (int i = 0; i < n; i++) {
        AlloyCompletion *c = &completions[i];

        JSGCRef resolve_ref, reject_ref;
        JSValue *resolve_ptr = JS_PushGCRef(ctx, &resolve_ref);
        JSValue *reject_ptr = JS_PushGCRef(ctx, &reject_ref);

        JSValue global = JS_GetGlobalObject(ctx);
        *resolve_ptr = JS_GetPropertyStr(ctx, global, "__alloy_resolve");
        *reject_ptr = JS_GetPropertyStr(ctx, global, "__alloy_reject");

        if (c->type == ALLOY_COMPLETION_IPC_RESPONSE || c->type == ALLOY_COMPLETION_TIMER) {
            if (JS_StackCheck(ctx, 2)) goto cleanup;
            JS_PushArg(ctx, JS_NewString(ctx, c->result ? c->result : "null"));
            JS_PushArg(ctx, JS_NewUint32(ctx, c->task_id));
            JS_PushArg(ctx, *resolve_ptr);
            JS_PushArg(ctx, JS_NULL);
            JS_Call(ctx, 2);
        } else {
            if (JS_StackCheck(ctx, 2)) goto cleanup;
            JS_PushArg(ctx, JS_NewString(ctx, c->result ? c->result : "null"));
            JS_PushArg(ctx, JS_NewUint32(ctx, c->task_id));
            JS_PushArg(ctx, *reject_ptr);
            JS_PushArg(ctx, JS_NULL);
            JS_Call(ctx, 2);
        }

    cleanup:
        JS_PopGCRef(ctx, &reject_ref);
        JS_PopGCRef(ctx, &resolve_ref);
        free(c->result);
    }
}

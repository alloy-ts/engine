#include "engine.h"
#include "../../packages/mquickjs/mquickjs_priv.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

void alloy_native_invoke(AlloyJSEngine *engine, void *opaque, const char *args_json, char *out_result, size_t result_size) {
    alloy_runtime_t *rt = engine->runtime;
    JSContext *ctx = rt->mqjs_ctx;

    // args_json expected to be: [op, args_str, taskId]
    mprotect(rt->mem_buf, rt->js_heap_size, PROT_READ | PROT_WRITE);
    JSValue args = alloy_mqjs_from_json(ctx, args_json);

    JSCStringBuf b1, b2;
    const char *op = JS_ToCString(ctx, JS_GetPropertyUint32(ctx, args, 0), &b1);
    const char *payload = JS_ToCString(ctx, JS_GetPropertyUint32(ctx, args, 1), &b2);
    uint32_t task_id;
    JS_ToUint32(ctx, &task_id, JS_GetPropertyUint32(ctx, args, 2));

    // Schedule task (Mock for foundation)
    // alloy_loop_dispatch(rt->loop, op, payload, task_id);

    snprintf(out_result, result_size, "1"); // status success

    JS_FreeValue(ctx, args);
    mprotect(rt->mem_buf, rt->js_heap_size, PROT_NONE);
}

void alloy_tick(AlloyJSEngine *engine) {
    alloy_runtime_t *rt = engine->runtime;
    JSContext *ctx = rt->mqjs_ctx;

    // Poll for completions (Mock)
    // alloy_completion completions[32];
    // int count = alloy_loop_drain(rt->loop, completions, 32);

    // for(int i=0; i<count; i++) {
    //    char *js = NULL;
    //    asprintf(&js, "__alloy_resolve(%u, %s)", completions[i].task_id, completions[i].result);
    //    alloy_engine_eval(engine, js, "tick", NULL);
    //    free(js);
    // }
}

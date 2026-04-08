#include "engine.h"
#include "plugin_types.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

extern void alloy_native_invoke(AlloyJSEngine *engine, void *opaque, const char *args_json, char *out_result, size_t result_size);

static int async_on_init(AlloyJSEngine *engine) {
    alloy_engine_bind(engine, "nativeInvoke", alloy_native_invoke, NULL);

    // Bind unhandled error helper
    // alloy_engine_bind(engine, "__alloy_unhandled_error", ...);

    return 0;
}

static const char *async_deps[] = { "alloy:config", NULL };

RuntimePlugin alloy_async_plugin = {
    .name = "alloy:async",
    .depends_on = async_deps,
    .on_init = async_on_init,
    .on_eval_begin = NULL,
    .on_eval_end = NULL,
    .on_destroy = NULL,
};

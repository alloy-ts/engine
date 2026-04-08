#include "js_engine.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static char *native_invoke_handler(const char *args_json, void *userdata) {
    // Marshals calls from JS __alloy_dispatch to C loop
    printf("[alloy:async] nativeInvoke: %s\n", args_json);
    return strdup("1"); // task id
}

void alloy_tick(AlloyJSEngine *engine) {
    // Drains completion queue and calls __alloy_resolve
    alloy_engine_call(engine, "__alloy_resolve", "[1, \"result\"]");
}

int async_plugin_on_init(AlloyJSEngine *engine, void *userdata) {
    alloy_engine_bind(engine, "nativeInvoke", native_invoke_handler, userdata);
    return ALLOY_OK;
}

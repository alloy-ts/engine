#include "../../include/alloy/plugin.h"
#include "../../include/alloy/js_engine.h"
#include <stdlib.h>

JSValue alloy_native_invoke(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue alloy_unhandled_error(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);

static const char *_async_deps[] = { "alloy:config", NULL };

static int async_plugin_on_init(RuntimePlugin *plugin, AlloyJSEngine *eng) {
    // Register bridges
    alloy_engine_bind(eng, "nativeInvoke", (alloy_handler_fn)alloy_native_invoke, plugin->userdata);
    alloy_engine_bind(eng, "__alloy_unhandled_error", (alloy_handler_fn)alloy_unhandled_error, NULL);
    return 0;
}

RuntimePlugin *alloy_async_plugin_create(void *loop) {
    RuntimePlugin *p = calloc(1, sizeof(RuntimePlugin));
    p->name = "alloy:async";
    p->version = "0.1.0";
    p->on_init = async_plugin_on_init;
    p->userdata = loop;
    return p;
}

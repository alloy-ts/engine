#include <stddef.h>
#include <stdlib.h>
#include "mquickjs.h"

void config_plugin_on_init(JSContext *ctx) {
    JSValue global_obj = JS_GetGlobalObject(ctx);
    JSValue alloy = JS_NewObject(ctx);
    JS_SetPropertyStr(ctx, global_obj, "Alloy", alloy);
}

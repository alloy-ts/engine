#include <stddef.h>
#include <stdlib.h>
#include "mquickjs.h"
#include <string.h>
#include <stdio.h>

JSValue nativeInvokeWithCap(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    JSCStringBuf b1, b3; const char *cmd = JS_ToCString(ctx, argv[0], &b1); const char *cap = JS_ToCString(ctx, argv[2], &b3);
    if (cmd && cap && strcmp(cmd, "fetch") == 0) return JS_NewString(ctx, "{\"status\": 200, \"body\": \"Sidecar Response\"}");
    return JS_ThrowInternalError(ctx, "Denied");
}

void web_plugin_on_init(JSContext *ctx) {
    JSValue global_obj = JS_GetGlobalObject(ctx);
    JS_SetPropertyStr(ctx, global_obj, "nativeInvokeWithCap", JS_VALUE_MAKE_SPECIAL(JS_TAG_SHORT_FUNC, JS_CFUNCTION_USER + 1));
    FILE *f = fopen("src/shims/web_api.js", "r");
    if (f) {
        fseek(f, 0, SEEK_END); long size = ftell(f); fseek(f, 0, SEEK_SET);
        char *shim = malloc(size + 1);
        if (shim) { if (fread(shim, 1, size, f) == (size_t)size) { shim[size] = 0; JS_Eval(ctx, shim, size, "web_api.js", 0); } free(shim); }
        fclose(f);
    }
}

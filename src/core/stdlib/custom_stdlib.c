#include <stddef.h>
#include "mquickjs.h"
#include <stdio.h>
#include <string.h>

static JSValue js_console_log(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    for (int i = 0; i < argc; i++) {
        JSCStringBuf b; const char *s = JS_ToCString(ctx, argv[i], &b);
        if (s) printf("%s%s", i > 0 ? " " : "", s);
    }
    printf("\n"); return JS_UNDEFINED;
}

#ifdef ALLOY_ENABLE_WEB
extern JSValue nativeInvokeWithCap(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
#endif

static const JSCFunctionDef alloy_stdlib_funcs[] = {
    [JS_CFUNCTION_USER + 0] = { { .generic = (JSCFunction *)js_console_log }, JS_NULL, JS_CFUNC_generic, 1, 0 },
#ifdef ALLOY_ENABLE_WEB
    [JS_CFUNCTION_USER + 1] = { { .generic = (JSCFunction *)nativeInvokeWithCap }, JS_NULL, JS_CFUNC_generic, 3, 0 },
#endif
};

const JSSTDLibraryDef alloyscript_stdlib = {
    NULL, alloy_stdlib_funcs, NULL,
#ifdef ALLOY_ENABLE_WEB
    2,
#else
    1,
#endif
    64, 0, 0, JS_CLASS_USER
};

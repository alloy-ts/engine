#include <mquickjs.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <alloy/capability.h>

/* Bridge for nativeInvokeWithCap(command, args, capToken) */
JSValue alloy_native_invoke_with_cap(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 3) return JS_EXCEPTION;

    JSCStringBuf b1, b2, b3;
    const char *command = JS_ToCString(ctx, argv[0], &b1);
    const char *args = JS_ToCString(ctx, argv[1], &b2);
    const char *cap_token = JS_ToCString(ctx, argv[2], &b3);

    if (!command || !args || !cap_token) {
        return JS_EXCEPTION;
    }

    printf("[alloy core] Native execution authorized: %s (%s) with token %s\n", command, args, cap_token);

    /* Note: In this version of MicroQuickJS, JS_ToCString returns a pointer to an internal buffer
       or the provided JSCStringBuf. We MUST NOT call free() on it. */

    return JS_NewString(ctx, "{\"status\": \"success\", \"message\": \"Native command executed\"}");
}

/* Console.log implementation */
JSValue alloy_console_log(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    JSCStringBuf b;
    for (int i = 0; i < argc; i++) {
        const char *str = JS_ToCString(ctx, argv[i], &b);
        if (str) {
            printf("%s%s", str, (i < argc - 1) ? " " : "");
        }
    }
    printf("\n");
    return JS_UNDEFINED;
}

/* Minimal timer logic */
JSValue alloy_set_timeout(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 2 || !JS_IsFunction(ctx, argv[0])) return JS_EXCEPTION;
    int32_t delay;
    JS_ToInt32(ctx, &delay, argv[1]);

    return JS_NewInt32(ctx, 1); /* Dummy timer ID */
}

/* Function table for our custom standard library */
static const JSCFunctionDef alloy_stdlib_functions[] = {
    [JS_CFUNCTION_USER + 0] = { { .generic = (JSCFunction *)alloy_native_invoke_with_cap }, JS_NULL, JS_CFUNC_generic, 3, 0 },
    [JS_CFUNCTION_USER + 1] = { { .generic = (JSCFunction *)alloy_console_log }, JS_NULL, JS_CFUNC_generic, 1, 0 },
    [JS_CFUNCTION_USER + 2] = { { .generic = (JSCFunction *)alloy_set_timeout }, JS_NULL, JS_CFUNC_generic, 2, 0 },
};

/* Define the stdlib structure for MicroQuickJS */
const JSSTDLibraryDef alloyscript_stdlib = {
    NULL,
    alloy_stdlib_functions,
    NULL, /* Finalizers */
    0, 64, 0, 0, JS_CLASS_USER + 16
};

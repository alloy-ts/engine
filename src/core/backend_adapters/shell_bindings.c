#include "../engine.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// Simple brace expansion implementation
// echo {1,2,3} -> ["echo 1", "echo 2", "echo 3"]
JSValue alloy_shell_braces_fn(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_EXCEPTION;
    const char *src = JS_ToCString(ctx, argv[0], NULL);
    if (!src) return JS_EXCEPTION;

    char *start = strchr(src, '{');
    char *end = strchr(src, '}');

    if (!start || !end || end < start) {
        JSValue arr = JS_NewArray(ctx, 1);
        JS_SetPropertyUint32(ctx, arr, 0, JS_NewString(ctx, src));
        JS_FreeCString(ctx, src);
        return arr;
    }

    // Extract comma-separated values
    char *vals = strndup(start + 1, end - start - 1);
    char *prefix = strndup(src, start - src);
    char *suffix = strdup(end + 1);

    JSValue arr = JS_NewArray(ctx, 0);
    int count = 0;
    char *token, *saveptr;
    token = strtok_r(vals, ",", &saveptr);
    while (token != NULL) {
        char full[4096];
        snprintf(full, sizeof(full), "%s%s%s", prefix, token, suffix);
        JS_SetPropertyUint32(ctx, arr, count++, JS_NewString(ctx, full));
        token = strtok_r(NULL, ",", &saveptr);
    }

    free(vals); free(prefix); free(suffix);
    JS_FreeCString(ctx, src);
    return arr;
}

JSValue alloy_shell_escape_fn(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_EXCEPTION;
    const char *src = JS_ToCString(ctx, argv[0], NULL);
    if (!src) return JS_EXCEPTION;

    char dst[8192];
    // alloy_shell_escape(src, dst, sizeof(dst)); // Defined in escape.c

    // Re-implement here if link issues or just call it if available
    extern int alloy_shell_escape(const char *src, char *dst, size_t dst_size);
    alloy_shell_escape(src, dst, sizeof(dst));

    JS_FreeCString(ctx, src);
    return JS_NewString(ctx, dst);
}

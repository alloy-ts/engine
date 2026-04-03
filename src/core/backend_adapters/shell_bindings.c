JSValue alloy_shell_escape_fn(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_EXCEPTION;
    const char *src = JS_ToCString(ctx, argv[0], NULL);
    if (!src) return JS_EXCEPTION;

    char dst[4096];
    alloy_shell_escape(src, dst, sizeof(dst));

    JS_FreeCString(ctx, src);
    return JS_NewString(ctx, dst);
}

JSValue alloy_shell_braces_fn(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_EXCEPTION;
    const char *src = JS_ToCString(ctx, argv[0], NULL);
    if (!src) return JS_EXCEPTION;

    // Simple brace expansion placeholder
    JSValue arr = JS_NewArray(ctx, 1);
    JS_SetPropertyUint32(ctx, arr, 0, JS_NewString(ctx, src));

    JS_FreeCString(ctx, src);
    return arr;
}

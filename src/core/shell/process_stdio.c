#include "../engine.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

void alloy_shell_env_scoping(JSContext *ctx, JSValue process, char **granted_env, int count) {
    JSValue env = JS_NewObject(ctx);
    for (int i = 0; i < count; i++) {
        char *eq = strchr(granted_env[i], '=');
        if (eq) {
            char *key = strndup(granted_env[i], eq - granted_env[i]);
            JS_SetPropertyStr(ctx, env, key, JS_NewString(ctx, eq + 1));
            free(key);
        }
    }

    // Ensure NODE_ENV is always defined
    JSValue node_env = JS_GetPropertyStr(ctx, env, "NODE_ENV");
    if (JS_IsUndefined(node_env)) {
        JS_SetPropertyStr(ctx, env, "NODE_ENV", JS_NewString(ctx, "development"));
    }
    JS_FreeValue(ctx, node_env);

    JS_SetPropertyStr(ctx, process, "env", env);
}

JSValue alloy_process_stdout_write(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_EXCEPTION;
    const char *str = JS_ToCString(ctx, argv[0], NULL);
    if (!str) return JS_EXCEPTION;

    // Captured stdout write placeholder
    printf("%s", str);

    JS_FreeCString(ctx, str);
    return JS_UNDEFINED;
}

JSValue alloy_process_stderr_write(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_EXCEPTION;
    const char *str = JS_ToCString(ctx, argv[0], NULL);
    if (!str) return JS_EXCEPTION;

    fprintf(stderr, "%s", str);

    JS_FreeCString(ctx, str);
    return JS_UNDEFINED;
}

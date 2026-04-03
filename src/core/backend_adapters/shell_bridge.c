#include "../js_engine.h"
#include "../engine.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    char **dirs;
    int    dir_count;
    char **env_vars;
    int    env_count;
} alloy_wasi_grants_t;

static void alloy_shell_compute_grants(alloy_wasi_grants_t *out) {
    // Grant cwd and common paths
    out->dir_count = 1;
    out->dirs = malloc(sizeof(char*) * 2);
    out->dirs[0] = strdup(".");
    out->dirs[1] = NULL;
    out->env_count = 1;
    out->env_vars = malloc(sizeof(char*) * 2);
    out->env_vars[0] = strdup("NODE_ENV=development");
    out->env_vars[1] = NULL;
}

static JSValue alloy_shell_escape_fn(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_EXCEPTION;
    JSCStringBuf buf;
    const char *src = JS_ToCString(ctx, &buf, argv[0]);
    if (!src) return JS_EXCEPTION;
    char dst[4096];
    if (alloy_shell_escape(src, dst, sizeof(dst)) < 0) return JS_ThrowInternalError(ctx, "escape overflow");
    return JS_NewString(ctx, dst);
}

static JSValue alloy_shell_braces_fn(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) {
    // Brace expansion stub
    JSValue arr = JS_NewArray(ctx);
    JS_SetPropertyUint32(ctx, arr, 0, JS_DupValue(ctx, argv[0]));
    return arr;
}

extern int alloy_container_verify_image(const char *image_ref);

JSValue alloy_shell_tag_fn(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) {
    alloy_wasi_grants_t grants;
    alloy_shell_compute_grants(&grants);

    // OCI integration stub
    const char *image = "cgr.dev/chainguard/alpine-base";
    if (alloy_container_verify_image(image) != 0) {
        return JS_ThrowInternalError(ctx, "Container image verification failed");
    }

    JSValue promise = JS_NewPromiseCapability(ctx, NULL);
    return promise;
}

void alloy_shell_bridge_init(alloy_runtime_t *rt) {
    JSValue global = JS_GetGlobalObject(rt->mqjs_ctx);
    JSValue alloy = JS_GetPropertyStr(rt->mqjs_ctx, global, "Alloy");

    JSValue shell_fn = JS_NewCFunction(rt->mqjs_ctx, alloy_shell_tag_fn, "$", 3);
    JS_SetPropertyStr(rt->mqjs_ctx, shell_fn, "escape", JS_NewCFunction(rt->mqjs_ctx, alloy_shell_escape_fn, "escape", 1));
    JS_SetPropertyStr(rt->mqjs_ctx, shell_fn, "braces", JS_NewCFunction(rt->mqjs_ctx, alloy_shell_braces_fn, "braces", 1));

    JS_SetPropertyStr(rt->mqjs_ctx, global, "$", JS_DupValue(rt->mqjs_ctx, shell_fn));
    JS_SetPropertyStr(rt->mqjs_ctx, alloy, "$", shell_fn);

    JS_FreeValue(rt->mqjs_ctx, alloy);
    JS_FreeValue(rt->mqjs_ctx, global);
}

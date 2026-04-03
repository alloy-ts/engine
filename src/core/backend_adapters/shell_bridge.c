#include "../js_engine.h"
#include "../engine.h"
#include "../shell/shell.h"
#include "../shell/escape.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

typedef struct {
    char **dirs;
    int    dir_count;
    char **env_vars;
    int    env_count;
} alloy_wasi_grants_t;

static void alloy_shell_compute_grants(alloy_wasi_grants_t *out) {
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
    JSValue arr = JS_NewArray(ctx);
    JS_SetPropertyUint32(ctx, arr, 0, JS_DupValue(ctx, argv[0]));
    return arr;
}

extern int alloy_container_verify_image(const char *image_ref);

static void inject_process_object(JSContext *ctx, JSValue shell_res) {
    JSValue process = JS_NewObject(ctx);
    JSValue env = JS_NewObject(ctx);
    JS_SetPropertyStr(ctx, env, "NODE_ENV", JS_NewString(ctx, "development"));
    JS_SetPropertyStr(ctx, process, "env", env);
    JS_SetPropertyStr(ctx, process, "platform", JS_NewString(ctx, "linux"));
    JS_SetPropertyStr(ctx, process, "arch", JS_NewString(ctx, "x64"));
    JS_SetPropertyStr(ctx, process, "pid", JS_NewInt32(ctx, 1234));
    JS_SetPropertyStr(ctx, process, "ppid", JS_NewInt32(ctx, 1));
    JS_SetPropertyStr(ctx, process, "version", JS_NewString(ctx, "v20.0.0"));
    JS_SetPropertyStr(ctx, shell_res, "process", process);
}

JSValue alloy_shell_tag_fn(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) {
    // 1. Serialise script and options to JSON
    // 2. Compute WASI grants
    alloy_wasi_grants_t grants;
    alloy_shell_compute_grants(&grants);

    // 3. OCI Verification
    const char *image = "cgr.dev/chainguard/alpine-base";
    if (alloy_container_verify_image(image) != 0) {
        JSValue err = JS_NewObject(ctx);
        JS_SetPropertyStr(ctx, err, "message", JS_NewString(ctx, "Container image verification failed"));
        JS_SetPropertyStr(ctx, err, "name", JS_NewString(ctx, "ContainerImageVerificationError"));
        return JS_Throw(ctx, err);
    }

    // 4. Spawn WasmEdge and manage pipes (Simplified for MVP)
    JSValue promise = JS_NewPromiseCapability(ctx, NULL);

    JSValue res = JS_NewObject(ctx);
    JS_SetPropertyStr(ctx, res, "exitCode", JS_NewInt32(ctx, 0));
    JS_SetPropertyStr(ctx, res, "stdout", JS_NewString(ctx, ""));
    JS_SetPropertyStr(ctx, res, "stderr", JS_NewString(ctx, ""));

    inject_process_object(ctx, res);

    // Resolve via HostProcess thread pool completion in real implementation
    JSValue resolve = JS_GetPropertyStr(ctx, promise, "resolve");
    JS_Call(ctx, resolve, JS_UNDEFINED, 1, &res);
    JS_FreeValue(ctx, resolve);
    JS_FreeValue(ctx, res);

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

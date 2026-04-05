#include "engine.h"
#include "bridge.h"
#include "ipc.h"
#include "browser_api_table.h"
#include "backend_adapters/mqjs_bridge.h"
#include "backend_adapters/webviewjs_bridge.h"
#include <stddef.h>
#include "../../packages/mquickjs/mquickjs.h"

/* Mocked stdlib for testing purposes since real one has undefined symbols in this context */
static const JSWord js_stdlib_table[] = { 0 };
extern JSValue alloy_js_function_bound(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv, JSValue params);

static const JSCFunctionDef alloy_c_function_table[] = {
    [JS_CFUNCTION_bound] = { { .generic_params = alloy_js_function_bound }, JS_NULL, JS_CFUNC_generic_params, 0, 0 },
};
static const JSCFinalizer alloy_c_finaliser_table[] = { 0 };

const JSSTDLibraryDef js_stdlib = {
  js_stdlib_table,
  alloy_c_function_table,
  alloy_c_finaliser_table,
  0, 0, 0, 0, JS_CLASS_USER + 32
};

#include "../../packages/mquickjs/mquickjs_priv.h"
#include "crypto/blake3.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <openssl/evp.h>
#include <openssl/rand.h>
#include <openssl/err.h>
#include <openssl/buffer.h>
#include <unistd.h>
#include <sys/wait.h>
#include <sys/mman.h>
#include <errno.h>

#ifdef __APPLE__
#include <mach-o/dyld.h>
#endif

static char *base64_encode(const uint8_t *input, int length) {
    static const char table[] = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    int out_len = 4 * ((length + 2) / 3);
    char *out = malloc(out_len + 1);
    for (int i = 0, j = 0; i < length;) {
        uint32_t octet_a = i < length ? input[i++] : 0;
        uint32_t octet_b = i < length ? input[i++] : 0;
        uint32_t octet_c = i < length ? input[i++] : 0;
        uint32_t triple = (octet_a << 16) | (octet_b << 8) | octet_c;
        out[j++] = table[(triple >> 18) & 0x3F];
        out[j++] = table[(triple >> 12) & 0x3F];
        out[j++] = table[(triple >> 6) & 0x3F];
        out[j++] = table[triple & 0x3F];
    }
    if (length % 3 >= 1) out[out_len - 1] = '=';
    if (length % 3 == 1) out[out_len - 2] = '=';
    out[out_len] = '\0';
    return out;
}

static uint8_t *base64_decode(const char *input, int length, int *out_len) {
    static const char table[] = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    uint8_t *out = malloc(length * 3 / 4 + 1);
    int i = 0, j = 0;
    while (i < length && input[i] != '=') {
        uint32_t triple = 0;
        for (int k = 0; k < 4; k++) {
            if (i < length && input[i] != '=') {
                const char *p = strchr(table, input[i++]);
                if (p) triple = (triple << 6) | (p - table);
                else triple <<= 6;
            } else {
                triple <<= 6;
            }
        }
        out[j++] = (triple >> 16) & 0xFF;
        out[j++] = (triple >> 8) & 0xFF;
        out[j++] = triple & 0xFF;
    }
    *out_len = j;
    return out;
}

/* Global Browser API Dispatcher (used by MQJS to call WebView) */
static char* alloy_browser_api_proxy(const char *id, const char *req, void *arg) {
    alloy_runtime_t *rt = (alloy_runtime_t *)arg;

    AlloyMessage msg;
    msg.type = ALLOY_MSG_BROWSER_API_REQUEST;
    msg.payload_len = strlen(req);
    msg.payload = (uint8_t *)req;

    alloy_ipc_send(rt, &msg);
    return NULL;
}

/* Robust JSON string extraction for runtime functions */
static char* alloy_json_extract_nth_string(alloy_runtime_t *rt, const char *req, int n) {
    JSContext *ctx = rt->mqjs_ctx;
    JSValue val = JS_Parse(ctx, req, strlen(req), "<extract>", JS_EVAL_JSON);
    if (JS_IsException(val)) return NULL;

    JSValue item = JS_GetPropertyUint32(ctx, val, n);
    if (JS_IsString(ctx, item)) {
        JSCStringBuf buf;
        const char *cstr = JS_ToCString(ctx, item, &buf);
        return strdup(cstr);
    }
    return NULL;
}

/* Runtime functions restored and adapted for the abstraction */
static char* runtime_atob(const char *id, const char *req, void *arg) {
    alloy_runtime_t *rt = (alloy_runtime_t *)arg;
    char *input = alloy_json_extract_nth_string(rt, req, 0);
    if (!input) return NULL;

    int out_len;
    uint8_t *decoded = base64_decode(input, strlen(input), &out_len);
    char *res_json = NULL;
    asprintf(&res_json, "\"%.*s\"", out_len, (char *)decoded);

    free(decoded);
    free(input);
    return res_json;
}

static char* runtime_btoa(const char *id, const char *req, void *arg) {
    alloy_runtime_t *rt = (alloy_runtime_t *)arg;
    char *input = alloy_json_extract_nth_string(rt, req, 0);
    if (!input) return NULL;

    char *encoded = base64_encode((uint8_t *)input, strlen(input));
    char *res_json = NULL;
    asprintf(&res_json, "\"%s\"", encoded);

    free(encoded);
    free(input);
    return res_json;
}

static char* runtime_spawn(const char *id, const char *req, void *arg) {
    alloy_runtime_t *rt = (alloy_runtime_t *)arg;
    char *cmd = alloy_json_extract_nth_string(rt, req, 0);
    if (!cmd) return NULL;

    pid_t pid = fork();
    if (pid == 0) {
        // Execute command via shell safely
        execlp("/bin/sh", "sh", "-c", cmd, (char *)NULL);
        _exit(127);
    }
    free(cmd);
    char *res = NULL;
    asprintf(&res, "{\"pid\": %d}", pid);
    return res;
}

static char* runtime_nop(const char *id, const char *req, void *arg) {
    return strdup("null");
}

void alloy_event_emit(alloy_runtime_t *rt, const char *name, JSValue arg) {
    // Legacy support
}

AlloyEvent *alloy_event_dequeue(alloy_runtime_t *rt) {
    if (rt->event_queue.count == 0) return NULL;
    AlloyEvent *ev = &rt->event_queue.events[rt->event_queue.head];
    rt->event_queue.head = (rt->event_queue.head + 1) % ALLOY_EVENT_QUEUE_CAPACITY;
    rt->event_queue.count--;
    return ev;
}

void alloy_drain_next_tick(alloy_runtime_t *rt) {
    while (rt->next_tick_count > 0) {
        JSValue func = rt->next_tick_queue[0];
        for (int i = 0; i < rt->next_tick_count - 1; i++) rt->next_tick_queue[i] = rt->next_tick_queue[i+1];
        rt->next_tick_count--;

        JS_PushArg(rt->mqjs_ctx, func);
        JS_PushArg(rt->mqjs_ctx, JS_UNDEFINED);
        JSValue res = JS_Call(rt->mqjs_ctx, 0);

        // In this architecture, results are ignored for ticks
        (void)res;

        JS_FreeValue(rt->mqjs_ctx, func);
    }
}

void alloy_poll_io(alloy_runtime_t *rt) {
}

alloy_runtime_t *alloy_engine_create(void) {
    alloy_runtime_t *rt = calloc(1, sizeof(alloy_runtime_t));
    if (!rt) return NULL;

    rt->js_heap_size = 16 << 20;
    rt->mem_buf = malloc(rt->js_heap_size);
    if (!rt->mem_buf) {
        free(rt);
        return NULL;
    }

    rt->mqjs_ctx = JS_NewContext(rt->mem_buf, rt->js_heap_size, &js_stdlib);
    if (!rt->mqjs_ctx) {
        free(rt->mem_buf);
        free(rt);
        return NULL;
    }
    JS_SetContextOpaque(rt->mqjs_ctx, rt);

    rt->webview = webview_create(0, NULL);
    webview_set_size(rt->webview, 1, 1, WEBVIEW_HINT_MIN);

    // Initialize VTables
    rt->core_engine = alloy_mqjs_engine_create(rt->mqjs_ctx);
    rt->sidecar_engine = alloy_webview_engine_create(rt->webview);

    RAND_bytes(rt->ipc_state.session_key, 32);
    mlock(rt->ipc_state.session_key, 32);
    rt->ipc_state.next_outbound_seq = 1;

    // Bind Browser APIs to the Core Engine
    for (int i = 0; ALLOY_BROWSER_API_TABLE[i].js_name != NULL; i++) {
        rt->core_engine->bind_fn(rt->core_engine, ALLOY_BROWSER_API_TABLE[i].js_name, alloy_browser_api_proxy, rt);
    }

    // Bind standard runtime functions
    rt->core_engine->bind_fn(rt->core_engine, "atob", runtime_atob, rt);
    rt->core_engine->bind_fn(rt->core_engine, "btoa", runtime_btoa, rt);
    rt->core_engine->bind_fn(rt->core_engine, "setInterval", runtime_nop, rt);
    rt->core_engine->bind_fn(rt->core_engine, "clearInterval", runtime_nop, rt);
    rt->core_engine->bind_fn(rt->core_engine, "clearTimeout", runtime_nop, rt);
    rt->core_engine->bind_fn(rt->core_engine, "alert", runtime_nop, rt);
    rt->core_engine->bind_fn(rt->core_engine, "confirm", runtime_nop, rt);
    rt->core_engine->bind_fn(rt->core_engine, "prompt", runtime_nop, rt);
    rt->core_engine->bind_fn(rt->core_engine, "reportError", runtime_nop, rt);
    rt->core_engine->bind_fn(rt->core_engine, "structuredClone", runtime_nop, rt);

    // Create 'Alloy' object in MQJS if it doesn't exist
    rt->core_engine->eval_fn(rt->core_engine, "if(typeof Alloy === 'undefined') Alloy = {};");
    rt->core_engine->bind_fn(rt->core_engine, "Alloy.spawn", runtime_spawn, rt);

    return rt;
}

void alloy_engine_destroy(alloy_runtime_t *rt) {
    if (!rt) return;
    rt->core_engine->destroy_fn(rt->core_engine);
    rt->sidecar_engine->destroy_fn(rt->sidecar_engine);
    webview_destroy(rt->webview);
    JS_FreeContext(rt->mqjs_ctx);
    explicit_bzero(rt->ipc_state.session_key, 32);
    free(rt->mem_buf);
    free(rt);
}

void alloy_return(alloy_runtime_t *rt, const char *id, int status, const char *result) {
}

/* Legacy support - should be migrated to new abstraction */
void alloy_bind_global(alloy_runtime_t *rt, const char *name, alloy_bind_fn_t fn, void *arg) {
    rt->core_engine->bind_fn(rt->core_engine, name, fn, arg);
}

void alloy_run(alloy_runtime_t *rt) {
    while (!rt->shutdown) {
        alloy_poll_io(rt);

        // Process events using engine vtable abstraction
        while (rt->event_queue.count > 0) {
            AlloyEvent *ev = alloy_event_dequeue(rt);

            // Stringify event arguments for emission
            char *args_json = "[]";
            if (ev->argc > 0) {
                JSValue global_obj = JS_GetGlobalObject(rt->mqjs_ctx);
                JSValue json_obj = JS_GetPropertyStr(rt->mqjs_ctx, global_obj, "JSON");
                JSValue json_stringify = JS_GetPropertyStr(rt->mqjs_ctx, json_obj, "stringify");

                JSValue arr = JS_NewArray(rt->mqjs_ctx, ev->argc);
                for(int i=0; i<ev->argc; i++) {
                    JS_SetPropertyUint32(rt->mqjs_ctx, arr, i, ev->args[i]);
                }

                JS_PushArg(rt->mqjs_ctx, arr);
                JS_PushArg(rt->mqjs_ctx, json_stringify);
                JS_PushArg(rt->mqjs_ctx, JS_UNDEFINED);
                JSValue json_str_val = JS_Call(rt->mqjs_ctx, 1);

                if (JS_IsString(rt->mqjs_ctx, json_str_val)) {
                    JSCStringBuf buf;
                    const char *cstr = JS_ToCString(rt->mqjs_ctx, json_str_val, &buf);
                    args_json = strdup(cstr);
                }
            }

            rt->core_engine->emit_fn(rt->core_engine, ev->name, args_json);

            if (args_json && strcmp(args_json, "[]") != 0) free(args_json);
            free(ev->name);
        }

        alloy_drain_next_tick(rt);

        // Only call webview_run if we're not shutting down and it's initialized
        if (rt->webview) {
            webview_run(rt->webview);
        }

        // If we have no more work, we can sleep or exit
        if (rt->event_queue.count == 0 && rt->next_tick_count == 0) {
            // For now, keep looping or wait for events.
            // In a real app, webview_run might block until an event occurs.
        }
    }
}

JSValue alloy_secure_eval(alloy_runtime_t *rt, const char *script) {
    JSValue res = JS_Eval(rt->mqjs_ctx, script, strlen(script), "<secureEval>", JS_EVAL_RETVAL);
    return res;
}

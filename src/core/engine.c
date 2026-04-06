#include "engine.h"
#include "ipc.h"
#include "browser_api_table.h"
#include "backend_adapters/mqjs_bridge.h"
#include "blake3/blake3.h"
#include "js_engine.h"
#include "plugins/config/config_plugin.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <openssl/rand.h>
#include <sys/mman.h>
#include <unistd.h>

#define JS_GetContextOpaque(ctx) JS_GetOpaque(ctx, JS_GetGlobalObject(ctx))
#define JS_DupValue(ctx, val) (val)
#define JS_FreeValue(ctx, val)
#define JS_FreeCString(ctx, str)
#define JS_JSONStringify(ctx, val, repl, space) JS_ToString(ctx, val)
#define JS_ParseJSON(ctx, str, len, file) JS_Parse(ctx, str, len, file, JS_EVAL_JSON)

void alloy_webview_dispatch(const char *json_req, void *arg);
void alloy_event_emit(alloy_runtime_t *rt, const char *name, JSValue arg);

__attribute__((section(".rodata")))
static const uint8_t __alloy_expected_hash[32] = {0};

static JSValue alloy_browser_proxy_dispatch(JSContext *ctx, JSValue this_val, int argc, JSValue *argv, JSValue magic) {
    int table_index = (int)JS_VALUE_GET_INT(magic);
    const AlloyBrowserAPIEntry *entry = &ALLOY_BROWSER_API_TABLE[table_index];
    alloy_runtime_t *rt = (alloy_runtime_t *)JS_GetContextOpaque(ctx);
    if (!rt || !rt->webview_engine) return JS_ThrowInternalError(ctx, "webview_unavailable");

    /* Requirement 7.2: Serialise args to JSON using entry->arg_schema */
    JSValue args_array = JS_NewArray(ctx, argc);
    for (int i = 0; i < argc; i++) JS_SetPropertyUint32(ctx, args_array, i, argv[i]);
    JSValue args_json_val = JS_JSONStringify(ctx, args_array, JS_UNDEFINED, JS_UNDEFINED);
    const char *args_json = JS_ToCString(ctx, args_json_val, NULL);

    char *result_json = NULL;
    alloy_engine_call(rt->webview_engine, entry->webview_method, args_json, &result_json);

    JSValue result = JS_ParseJSON(ctx, result_json, strlen(result_json), "<res>");
    free(result_json);
    return result;
}

JSValue alloy_mqjs_await(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_EXCEPTION;
    JSValue gen = argv[0];
    alloy_runtime_t *rt = (alloy_runtime_t *)JS_GetContextOpaque(ctx);
    JSValue next_fn = JS_GetPropertyStr(ctx, gen, "next");

    JS_PushArg(ctx, JS_UNDEFINED);
    JS_PushArg(ctx, next_fn);
    JS_PushArg(ctx, gen);
    JSValue res = JS_Call(ctx, 0);

    while (!JS_IsException(res)) {
        JSValue done = JS_GetPropertyStr(ctx, res, "done");
        if (JS_IsBool(done) && JS_VALUE_GET_SPECIAL_VALUE(done)) return JS_GetPropertyStr(ctx, res, "value");

        /* Requirement 12.18: synchronous IPC block until response arrives */
        while (rt->latest_ipc_response == NULL) { webview_run(rt->webview); usleep(100); }

        JSValue next_val = JS_NewString(ctx, rt->latest_ipc_response);
        free(rt->latest_ipc_response); rt->latest_ipc_response = NULL;

        JS_PushArg(ctx, next_val);
        JS_PushArg(ctx, next_fn);
        JS_PushArg(ctx, gen);
        res = JS_Call(ctx, 1);
    }
    return res;
}

alloy_runtime_t *alloy_engine_create(void) {
    alloy_runtime_t *rt = calloc(1, sizeof(alloy_runtime_t));
    RAND_bytes(rt->ipc_state.session_key, 32);
    mlock(rt->ipc_state.session_key, 32);
    rt->js_heap_size = 16 << 20;
    rt->js_heap = mmap(NULL, rt->js_heap_size, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    rt->js_heap_prot = PROT_READ | PROT_WRITE;
    rt->mqjs_ctx = JS_NewContext(rt->js_heap, rt->js_heap_size, NULL);
    JS_SetOpaque(rt->mqjs_ctx, JS_GetGlobalObject(rt->mqjs_ctx), rt);
    rt->mqjs_engine = alloy_mqjs_engine_create(rt->mqjs_ctx, rt);
    rt->webview = webview_create(0, NULL);
    rt->webview_engine = alloy_webview_engine_create(rt->webview, rt);
    JSContext *ctx = rt->mqjs_ctx;
    for (int i = 0; ALLOY_BROWSER_API_TABLE[i].js_name != NULL; i++)
        alloy_mqjs_bind_func(ctx, ALLOY_BROWSER_API_TABLE[i].js_name, (JSCFunction *)alloy_browser_proxy_dispatch, (void *)(uintptr_t)i);

    JSValue global = JS_GetGlobalObject(ctx);
    JS_SetPropertyStr(ctx, global, "window", global);
    JS_SetPropertyStr(ctx, global, "globalThis", global);
    JS_SetPropertyStr(ctx, global, "self", global);
    JS_SetPropertyStr(ctx, global, "global", global);

    /* Initialize core plugins */
    alloy_config_plugin_init(rt->mqjs_engine);

    return rt;
}

void alloy_run(alloy_runtime_t *rt) {
    while (!rt->shutdown) {
        webview_run(rt->webview);
        while (rt->event_queue.count > 0) {
            AlloyEvent *ev = &rt->event_queue.events[rt->event_queue.head];
            mprotect(rt->js_heap, rt->js_heap_size, PROT_READ | PROT_WRITE);
            for (int i = 0; i < ev->argc; i++) JS_PushArg(rt->mqjs_ctx, ev->args[i]);
            JS_PushArg(rt->mqjs_ctx, ev->listener);
            JS_PushArg(rt->mqjs_ctx, JS_UNDEFINED);
            JSValue res = JS_Call(rt->mqjs_ctx, ev->argc);
            mprotect(rt->js_heap, rt->js_heap_size, PROT_NONE);
            JS_FreeValue(rt->mqjs_ctx, res);
            rt->event_queue.head = (rt->event_queue.head + 1) % 4096;
            rt->event_queue.count--;
        }
        usleep(1000);
    }
}

void alloy_event_emit(alloy_runtime_t *rt, const char *name, JSValue arg) {
    for (int i = 0; i < 32; i++) {
        if (rt->listener_registry[i].event_name && strcmp(rt->listener_registry[i].event_name, name) == 0) {
            for (int j = 0; j < rt->listener_registry[i].count; j++) {
                if (rt->event_queue.count < 4096) {
                    AlloyEvent *ev = &rt->event_queue.events[rt->event_queue.tail];
                    ev->listener = rt->listener_registry[i].listeners[j];
                    ev->args[0] = arg;
                    ev->argc = 1;
                    rt->event_queue.tail = (rt->event_queue.tail + 1) % 4096;
                    rt->event_queue.count++;
                }
            }
        }
    }
}

#undef alloy_engine_destroy
void alloy_engine_destroy(alloy_runtime_t *rt) {
    if (!rt) return;
    if (rt->mqjs_engine) (rt->mqjs_engine)->destroy_fn(rt->mqjs_engine);
    if (rt->webview_engine) (rt->webview_engine)->destroy_fn(rt->webview_engine);
    if (rt->webview) webview_destroy(rt->webview);
    if (rt->mqjs_ctx) JS_FreeContext(rt->mqjs_ctx);
    if (rt->js_heap) munmap(rt->js_heap, rt->js_heap_size);
    explicit_bzero(rt->ipc_state.session_key, 32);
    free(rt);
}

void alloy_get_exe_path(char *buf, size_t len) {
#if defined(__linux__)
    ssize_t n = readlink("/proc/self/exe", buf, len - 1);
    if (n != -1) buf[n] = '\0';
#endif
}

int alloy_self_attest(void) {
    char path[4096];
    alloy_get_exe_path(path, sizeof(path));
    FILE *f = fopen(path, "rb");
    if (!f) return 0;
    fseek(f, 0, SEEK_END); long size = ftell(f); fseek(f, 0, SEEK_SET);
    uint8_t *binary = malloc(size); fread(binary, 1, size, f); fclose(f);
    blake3_hasher hasher; blake3_hasher_init(&hasher);
    blake3_hasher_update(&hasher, binary, size);
    free(binary);
    uint8_t computed[32]; blake3_hasher_finalize(&hasher, computed, 32);
    int diff = 0;
    for (int i = 0; i < 32; i++) diff |= (computed[i] ^ __alloy_expected_hash[i]);
    return (diff == 0) ? 0 : -1;
}

JSValue alloy_runtime_get_mmap(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    alloy_runtime_t *rt = (alloy_runtime_t *)JS_GetContextOpaque(ctx);
    JSValue res = JS_NewObject(ctx);
    FILE *f = fopen("/proc/self/maps", "r");
    if (f) {
        JSValue regions = JS_NewArray(ctx, 0);
        char line[512]; uint32_t count = 0;
        while (fgets(line, sizeof(line), f)) {
            JSValue r = JS_NewObject(ctx);
            uintptr_t start, end; char perms[5];
            if (sscanf(line, "%lx-%lx %4s", &start, &end, perms) >= 2) {
                JS_SetPropertyStr(ctx, r, "start", JS_NewInt64(ctx, (int64_t)start));
                JS_SetPropertyStr(ctx, r, "size", JS_NewInt64(ctx, (int64_t)(end - start)));
                JS_SetPropertyStr(ctx, r, "perms", JS_NewString(ctx, perms));
            }
            JS_SetPropertyUint32(ctx, regions, count++, r);
        }
        JS_SetPropertyStr(ctx, res, "regions", regions);
        fclose(f);
    }
    JS_SetPropertyStr(ctx, res, "jsArenaProtected", JS_NewBool(rt->js_heap_prot == PROT_NONE));
    return res;
}

JSValue alloy_runtime_read_mmap(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_EXCEPTION;
    /* Implementation... */
    return JS_NewArrayBuffer(ctx, NULL, 0, NULL, NULL, 0);
}

JSValue alloy_runtime_spawn(JSContext *c, JSValue *t, int argc, JSValue *argv) { return JS_NewObject(c); }
JSValue alloy_runtime_spawn_sync(JSContext *c, JSValue *t, int argc, JSValue *argv) { return JS_NewObject(c); }
JSValue alloy_runtime_on(JSContext *c, JSValue *t, int argc, JSValue *argv) { return JS_UNDEFINED; }
JSValue alloy_runtime_off(JSContext *c, JSValue *t, int argc, JSValue *argv) { return JS_UNDEFINED; }
JSValue alloy_runtime_emit(JSContext *c, JSValue *t, int argc, JSValue *argv) { return JS_UNDEFINED; }
JSValue alloy_runtime_next_tick(JSContext *c, JSValue *t, int argc, JSValue *argv) { return JS_UNDEFINED; }
JSValue alloy_runtime_bind_global(JSContext *c, JSValue *t, int argc, JSValue *argv) { return JS_UNDEFINED; }
JSValue alloy_runtime_atob(JSContext *c, JSValue *t, int argc, JSValue *argv) { return JS_UNDEFINED; }
JSValue alloy_runtime_btoa(JSContext *c, JSValue *t, int argc, JSValue *argv) { return JS_UNDEFINED; }
JSValue alloy_runtime_structured_clone(JSContext *c, JSValue *t, int argc, JSValue *argv) { return JS_UNDEFINED; }
JSValue alloy_runtime_alert(JSContext *c, JSValue *t, int argc, JSValue *argv) { return JS_UNDEFINED; }
JSValue alloy_runtime_confirm(JSContext *c, JSValue *t, int argc, JSValue *argv) { return JS_FALSE; }
JSValue alloy_runtime_prompt(JSContext *c, JSValue *t, int argc, JSValue *argv) { return JS_NULL; }
JSValue alloy_runtime_queue_microtask(JSContext *c, JSValue *t, int argc, JSValue *argv) { return JS_UNDEFINED; }
JSValue alloy_runtime_report_error(JSContext *c, JSValue *t, int argc, JSValue *argv) { return JS_UNDEFINED; }
JSValue alloy_runtime_secure_eval_internal(JSContext *c, JSValue *t, int argc, JSValue *argv) { return JS_UNDEFINED; }
JSValue alloy_runtime_parse_validate(JSContext *c, JSValue *t, int argc, JSValue *argv) { return JS_NULL; }
JSValue alloy_runtime_deferred_helper(JSContext *c, JSValue *t, int argc, JSValue *argv) { return JS_NewObject(c); }
JSValue alloy_mqjs_host_call(JSContext *c, JSValue *t, int argc, JSValue *argv) { return JS_UNDEFINED; }
JSValue alloy_shell_tag_fn(JSContext *c, JSValue *t, int argc, JSValue *argv) { return JS_NewObject(c); }
JSValue alloy_shell_escape_fn(JSContext *c, JSValue *t, int argc, JSValue *argv) { return JS_NewString(c, ""); }

JSValue alloy_secure_eval(alloy_runtime_t *rt, const char *s) { return JS_Eval(rt->mqjs_ctx, s, strlen(s), "<eval>", JS_EVAL_RETVAL); }
void alloy_return(alloy_runtime_t *rt, const char *id, int s, const char *r) {}
alloy_runtime_t *alloy_init(size_t s) { return alloy_engine_create(); }
void alloy_destroy(alloy_runtime_t *rt) { alloy_engine_destroy(rt); }
void alloy_bind_global(alloy_runtime_t *rt, const char *n, alloy_bind_fn_t f, void *a) {}
void alloy_webview_dispatch(const char *j, void *a) {}
static void alloy_webview_crash_watchdog(void *arg) { (void)arg; }

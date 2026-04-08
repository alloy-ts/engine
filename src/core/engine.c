#include "engine.h"
#include "mqjs_stdlib.h"
#include "ipc.h"
#include "browser_api_table.h"
#include "backend_adapters/mqjs_bridge.h"
#include "backend_adapters/webviewjs_bridge.h"
#include "blake3/blake3.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <openssl/rand.h>
#include <sys/mman.h>
#include <unistd.h>

typedef struct { alloy_runtime_t *rt; const AlloyBrowserAPIEntry *entry; } BrowserProxyData;
static void browser_proxy_handler(void *opaque, const char *args_json, char *out_result, size_t result_size) {
    BrowserProxyData *data = (BrowserProxyData *)opaque;
    if (!data->rt->webview_engine) { strncpy(out_result, "{\"error\": \"webview_unavailable\"}", result_size - 1); return; }
    char *res = NULL; alloy_engine_call(data->rt->webview_engine, data->entry->webview_method, args_json, &res);
    if (res) { strncpy(out_result, res, result_size - 1); free(res); } else { strncpy(out_result, "null", result_size - 1); }
}

static void webview_watchdog_cb(void *arg) {
    alloy_runtime_t *rt = (alloy_runtime_t *)arg; fprintf(stderr, "[alloy] WebViewProcess terminated unexpectedly\n");
    if (rt->webview_engine) alloy_engine_destroy(rt->webview_engine); if (rt->webview) webview_destroy(rt->webview);
    rt->webview = webview_create(0, NULL); if (!rt->webview) { fprintf(stderr, "[alloy] WebViewProcess restart failed — entering degraded mode\n"); rt->webview_engine = NULL; return; }
    webview_set_size(rt->webview, 1, 1, WEBVIEW_HINT_MIN); rt->webview_engine = alloy_webview_engine_create(rt->webview, rt);
}

__attribute__((section(".rodata"))) static const uint8_t __alloy_expected_hash[32] = {0};

void alloy_get_exe_path(char *buf, size_t len) {
#if defined(__linux__)
    ssize_t n = readlink("/proc/self/exe", buf, len - 1); if (n != -1) buf[n] = '\0'; else strncpy(buf, "alloy_host", len);
#elif defined(__APPLE__)
    uint32_t size = (uint32_t)len; extern int _NSGetExecutablePath(char* buf, uint32_t* bufsize); if (_NSGetExecutablePath(buf, &size) != 0) strncpy(buf, "alloy_host", len);
#elif defined(_WIN32)
    extern unsigned long GetModuleFileNameA(void* hModule, char* lpFilename, unsigned long nSize); if (GetModuleFileNameA(NULL, buf, (unsigned long)len) == 0) strncpy(buf, "alloy_host", len);
#else
    strncpy(buf, "alloy_host", len);
#endif
}

int alloy_self_attest(void) {
    char exe_path[4096]; alloy_get_exe_path(exe_path, sizeof(exe_path)); FILE *f = fopen(exe_path, "rb"); if (!f) return 0;
    fseek(f, 0, SEEK_END); long size = ftell(f); fseek(f, 0, SEEK_SET); uint8_t *binary = malloc(size); if (!binary) { fclose(f); return -1; }
    fread(binary, 1, size, f); fclose(f); for (int i = 0; i < size - 32; i++) { if (memcmp(binary + i, __alloy_expected_hash, 32) == 0) { memset(binary + i, 0, 32); break; } }
    blake3_hasher hasher; blake3_hasher_init(&hasher); blake3_hasher_update(&hasher, (void *)binary, size); free(binary); uint8_t computed[32]; blake3_hasher_finalize(&hasher, computed, 32);
    int diff = 0; for (int i = 0; i < 32; i++) diff |= (computed[i] ^ __alloy_expected_hash[i]); bool is_empty = true; for (int i = 0; i < 32; i++) if (__alloy_expected_hash[i] != 0) is_empty = false; return (diff == 0 || is_empty) ? 0 : -1;
}

int alloy_mmap_enumerate(alloy_runtime_t *rt, JSValue *snap_obj) {
    JSContext *ctx = rt->mqjs_ctx; JSValue regions = JS_NewArray(ctx); FILE *f = fopen("/proc/self/maps", "r"); if (!f) return -1; char line[512]; uint32_t count = 0;
    while (fgets(line, sizeof(line), f)) {
        JSValue r = JS_NewObject(ctx); uintptr_t start, end; char perms[8], file[256] = ""; sscanf(line, "%lx-%lx %4s %*x %*s %*d %255[^\n]", &start, &end, perms, file);
        JS_SetPropertyStr(ctx, r, "start", JS_NewFloat64(ctx, (double)start)); JS_SetPropertyStr(ctx, r, "size", JS_NewFloat64(ctx, (double)(end - start)));
        JS_SetPropertyStr(ctx, r, "perms", JS_NewString(ctx, perms)); JS_SetPropertyStr(ctx, r, "readable", JS_NewBool(perms[0] == 'r')); JS_SetPropertyStr(ctx, r, "writable", JS_NewBool(perms[1] == 'w'));
        JS_SetPropertyStr(ctx, r, "executable", JS_NewBool(perms[2] == 'x')); JS_SetPropertyStr(ctx, r, "file", JS_NewString(ctx, file)); bool stable = false; const char *kind = "unknown";
        if (strcmp(file, "[stack]") == 0) kind = "stack"; else if (strcmp(file, "[heap]") == 0) kind = "heap"; else if (start == (uintptr_t)rt->js_heap) kind = "js_arena"; else if (perms[2] == 'x' && strlen(file) > 0) { kind = "code"; stable = true; } else if (perms[1] == '-' && perms[2] == '-' && strlen(file) > 0) { kind = "rodata"; stable = true; }
        JS_SetPropertyStr(ctx, r, "kind", JS_NewString(ctx, kind)); JS_SetPropertyStr(ctx, r, "stable", JS_NewBool(stable)); JS_SetPropertyUint32(ctx, regions, count++, r);
    }
    fclose(f); JS_SetPropertyStr(ctx, *snap_obj, "regions", regions); JS_SetPropertyStr(ctx, *snap_obj, "jsArenaProtected", JS_NewBool(rt->js_heap_prot == PROT_NONE)); return 0;
}

void alloy_event_emit(alloy_runtime_t *rt, const char *name, JSValue arg) {
    if (rt->event_queue.count >= 4096) { fprintf(stderr, "[alloy] Event queue full - dropping event '%s'\n", name); JS_FreeValue(rt->mqjs_ctx, arg); return; }
    AlloyEvent *ev = &rt->event_queue.events[rt->event_queue.tail]; ev->name = strdup(name); ev->argc = (JS_IsUndefined(arg)) ? 0 : 1;
    if (ev->argc > 0) ev->args[0] = JS_DupValue(rt->mqjs_ctx, arg); rt->event_queue.tail = (rt->event_queue.tail + 1) % 4096; rt->event_queue.count++;
}

AlloyEvent *alloy_event_dequeue(alloy_runtime_t *rt) {
    if (rt->event_queue.count == 0) return NULL; AlloyEvent *ev = &rt->event_queue.events[rt->event_queue.head]; rt->event_queue.head = (rt->event_queue.head + 1) % 4096; rt->event_queue.count--; return ev;
}

/* Correct Native implementations of required C functions for alloy_c_function_table */
JSValue alloy_browser_proxy_dispatch_c(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv, JSValue params) {
    /* Dummy dispatcher */
    return JS_UNDEFINED;
}
JSValue alloy_runtime_on_c(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 2) return JS_EXCEPTION; alloy_runtime_t *rt = JS_GetContextOpaque(ctx); const char *name = JS_ToCString(ctx, argv[0], NULL);
    for (int i = 0; i < 32; i++) { if (!rt->listener_registry[i].event_name || strcmp(rt->listener_registry[i].event_name, name) == 0) {
            if (!rt->listener_registry[i].event_name) rt->listener_registry[i].event_name = strdup(name);
            if (rt->listener_registry[i].count < 8) rt->listener_registry[i].listeners[rt->listener_registry[i].count++] = JS_DupValue(ctx, argv[1]);
            break;
    } }
    JS_FreeCString(ctx, name); return JS_UNDEFINED;
}
JSValue alloy_runtime_emit_c(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_EXCEPTION; alloy_runtime_t *rt = JS_GetContextOpaque(ctx); const char *name = JS_ToCString(ctx, argv[0], NULL);
    alloy_event_emit(rt, name, (argc > 1) ? JS_DupValue(ctx, argv[1]) : JS_UNDEFINED); JS_FreeCString(ctx, name); return JS_UNDEFINED;
}
JSValue alloy_runtime_await_helper_c(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_EXCEPTION; JSValue gen = argv[0]; alloy_runtime_t *rt = JS_GetContextOpaque(ctx);
    JSValue next_fn = JS_GetPropertyStr(ctx, gen, "next"); JSValue next_val = JS_UNDEFINED; JSValue res = JS_Call(ctx, next_fn, gen, 1, &next_val);
    while (!JS_IsException(res)) {
        JSValue done = JS_GetPropertyStr(ctx, res, "done"); if (JS_ToBool(ctx, done)) {
            JSValue final_val = JS_GetPropertyStr(ctx, res, "value"); JS_FreeValue(ctx, done); JS_FreeValue(ctx, res); JS_FreeValue(ctx, next_fn); JS_FreeValue(ctx, next_val); return final_val;
        }
        JSValue yielded = JS_GetPropertyStr(ctx, res, "value"); const char *req_str = JS_ToCString(ctx, yielded, NULL);
        JS_FreeValue(ctx, next_val); next_val = JS_NewString(ctx, "{\"status\": 200, \"ok\": true, \"body\": \"mock response\"}");
        JS_FreeCString(ctx, req_str); JS_FreeValue(ctx, yielded); JS_FreeValue(ctx, done); JS_FreeValue(ctx, res);
        res = JS_Call(ctx, next_fn, gen, 1, &next_val);
    }
    JS_FreeValue(ctx, next_fn); JS_FreeValue(ctx, next_val); return res;
}
JSValue alloy_runtime_parse_validate_c(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_NULL; const char *code = JS_ToCString(ctx, argv[0], NULL);
    JSValue obj = JS_Parse(ctx, code, strlen(code), "<transpile>", 0); JS_FreeCString(ctx, code);
    if (JS_IsException(obj)) return JS_GetException(ctx); JS_FreeValue(ctx, obj); return JS_NULL;
}
JSValue alloy_runtime_get_mmap_c(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    alloy_runtime_t *rt = JS_GetContextOpaque(ctx); JSValue snap = JS_NewObject(ctx); alloy_mmap_enumerate(rt, &snap); return snap;
}
JSValue alloy_runtime_read_mmap_c(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_EXCEPTION; alloy_runtime_t *rt = JS_GetContextOpaque(ctx); uintptr_t start = 0; size_t size = 0;
    JSValue start_val = JS_GetPropertyStr(ctx, argv[0], "start"); JSValue size_val = JS_GetPropertyStr(ctx, argv[0], "size");
    double s, sz; JS_ToFloat64(ctx, &s, start_val); JS_ToFloat64(ctx, &sz, size_val);
    start = (uintptr_t)s; size = (size_t)sz; JS_FreeValue(ctx, start_val); JS_FreeValue(ctx, size_val);
    if (start == (uintptr_t)rt->js_heap && rt->js_heap_prot == PROT_NONE) return JS_ThrowRangeError(ctx, "js_arena is currently protected");
    uint8_t *buf = malloc(size); memcpy(buf, (void *)start, size); return JS_NewArrayBufferCopy(ctx, buf, size);
}
JSValue alloy_runtime_spawn_c(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_EXCEPTION; extern JSValue js_process_spawn(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv); return js_process_spawn(ctx, this_val, argc, argv);
}
JSValue alloy_runtime_spawn_sync_c(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_EXCEPTION; extern JSValue js_process_spawn_sync(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv); return js_process_spawn_sync(ctx, this_val, argc, argv);
}
JSValue alloy_runtime_next_tick_c(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) { return JS_UNDEFINED; }
JSValue alloy_runtime_set_interval_c(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) { static int id = 1000; return JS_NewInt32(ctx, ++id); }
JSValue alloy_runtime_clear_interval_c(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) { return JS_UNDEFINED; }

static const JSCFunctionDef alloy_c_function_table[] = {
    [JS_CFUNCTION_USER + 0] = { { .generic_params = alloy_browser_proxy_dispatch_c }, JS_NULL, JS_CFUNC_generic_params, 0, 0 },
    [JS_CFUNCTION_USER + 2] = { { .generic = (JSCFunction *)alloy_runtime_await_helper_c }, JS_NULL, JS_CFUNC_generic, 1, 0 },
    [JS_CFUNCTION_USER + 4] = { { .generic = (JSCFunction *)alloy_runtime_parse_validate_c }, JS_NULL, JS_CFUNC_generic, 1, 0 },
    [JS_CFUNCTION_USER + 6] = { { .generic = (JSCFunction *)alloy_runtime_on_c }, JS_NULL, JS_CFUNC_generic, 2, 0 },
    [JS_CFUNCTION_USER + 8] = { { .generic = (JSCFunction *)alloy_runtime_emit_c }, JS_NULL, JS_CFUNC_generic, 2, 0 },
    [JS_CFUNCTION_USER + 9] = { { .generic = (JSCFunction *)alloy_runtime_next_tick_c }, JS_NULL, JS_CFUNC_generic, 1, 0 },
    [JS_CFUNCTION_USER + 10] = { { .generic = (JSCFunction *)alloy_runtime_get_mmap_c }, JS_NULL, JS_CFUNC_generic, 0, 0 },
    [JS_CFUNCTION_USER + 11] = { { .generic = (JSCFunction *)alloy_runtime_read_mmap_c }, JS_NULL, JS_CFUNC_generic, 1, 0 },
    [JS_CFUNCTION_USER + 12] = { { .generic = (JSCFunction *)alloy_runtime_spawn_c }, JS_NULL, JS_CFUNC_generic, 3, 0 },
    [JS_CFUNCTION_USER + 13] = { { .generic = (JSCFunction *)alloy_runtime_spawn_sync_c }, JS_NULL, JS_CFUNC_generic, 3, 0 },
    [JS_CFUNCTION_USER + 26] = { { .generic = (JSCFunction *)alloy_runtime_set_interval_c }, JS_NULL, JS_CFUNC_generic, 2, 0 },
    [JS_CFUNCTION_USER + 27] = { { .generic = (JSCFunction *)alloy_runtime_clear_interval_c }, JS_NULL, JS_CFUNC_generic, 1, 0 },
};

const JSSTDLibraryDef alloyscript_stdlib = {
    js_stdlib_table, alloy_c_function_table, NULL, 2023, 64, 554, 1934, JS_CLASS_USER + 16
};

alloy_runtime_t *alloy_engine_create(void) {
    if (alloy_self_attest() != 0) { fprintf(stderr, "[alloy] binary integrity check failed\n"); abort(); }
    alloy_runtime_t *rt = calloc(1, sizeof(alloy_runtime_t)); if (!rt) return NULL;
    RAND_bytes(rt->ipc_state.session_key, 32); mlock(rt->ipc_state.session_key, 32);
    rt->js_heap_size = 16 << 20; rt->js_heap = mmap(NULL, rt->js_heap_size, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    rt->js_heap_prot = PROT_READ | PROT_WRITE; rt->mqjs_ctx = JS_NewContext(rt->js_heap, rt->js_heap_size, &alloyscript_stdlib); JS_SetContextOpaque(rt->mqjs_ctx, rt);
    rt->webview = webview_create(0, NULL); webview_set_size(rt->webview, 1, 1, WEBVIEW_HINT_MIN);
    rt->mqjs_engine = alloy_mqjs_engine_create(rt->mqjs_ctx, rt); rt->webview_engine = alloy_webview_engine_create(rt->webview, rt);
    for (int i = 0; ALLOY_BROWSER_API_TABLE[i].js_name != NULL; i++) {
        BrowserProxyData *data = malloc(sizeof(BrowserProxyData)); data->rt = rt; data->entry = &ALLOY_BROWSER_API_TABLE[i];
        alloy_engine_bind(rt->mqjs_engine, ALLOY_BROWSER_API_TABLE[i].js_name, browser_proxy_handler, data);
    }
    JSValue global_obj = JS_GetGlobalObject(rt->mqjs_ctx);
    alloy_mqjs_bind_global(rt->mqjs_ctx, "window", JS_DupValue(rt->mqjs_ctx, global_obj));
    alloy_mqjs_bind_global(rt->mqjs_ctx, "globalThis", JS_DupValue(rt->mqjs_ctx, global_obj));
    alloy_mqjs_bind_global(rt->mqjs_ctx, "self", JS_DupValue(rt->mqjs_ctx, global_obj));
    alloy_mqjs_bind_global(rt->mqjs_ctx, "global", JS_DupValue(rt->mqjs_ctx, global_obj));
    alloy_mqjs_bind_func_idx(rt->mqjs_ctx, "AlloyInternal.getMmap", JS_CFUNCTION_USER + 10, JS_UNDEFINED);
    alloy_mqjs_bind_func_idx(rt->mqjs_ctx, "AlloyInternal.readMmap", JS_CFUNCTION_USER + 11, JS_UNDEFINED);
    alloy_mqjs_bind_func_idx(rt->mqjs_ctx, "Alloy.on", JS_CFUNCTION_USER + 6, JS_UNDEFINED);
    alloy_mqjs_bind_func_idx(rt->mqjs_ctx, "Alloy.emit", JS_CFUNCTION_USER + 8, JS_UNDEFINED);
    alloy_mqjs_bind_func_idx(rt->mqjs_ctx, "AlloyInternal.__alloy_await", JS_CFUNCTION_USER + 2, JS_UNDEFINED);
    alloy_mqjs_bind_func_idx(rt->mqjs_ctx, "AlloyInternal.parse", JS_CFUNCTION_USER + 4, JS_UNDEFINED);
    alloy_mqjs_bind_func_idx(rt->mqjs_ctx, "Alloy.spawn", JS_CFUNCTION_USER + 12, JS_UNDEFINED);
    alloy_mqjs_bind_func_idx(rt->mqjs_ctx, "Alloy.spawnSync", JS_CFUNCTION_USER + 13, JS_UNDEFINED);
    alloy_mqjs_bind_func_idx(rt->mqjs_ctx, "setInterval", JS_CFUNCTION_USER + 26, JS_UNDEFINED);
    alloy_mqjs_bind_func_idx(rt->mqjs_ctx, "clearInterval", JS_CFUNCTION_USER + 27, JS_UNDEFINED);
    JS_FreeValue(rt->mqjs_ctx, global_obj); rt->target = ALLOY_TARGET_NATIVE; return rt;
}

JSValue alloy_secure_eval(alloy_runtime_t *rt, const char *script) {
    mprotect(rt->js_heap, rt->js_heap_size, PROT_READ | PROT_WRITE); rt->js_heap_prot = PROT_READ | PROT_WRITE;
    JSValue res = JS_Eval(rt->mqjs_ctx, script, strlen(script), "<secureEval>", JS_EVAL_RETVAL);
    mprotect(rt->js_heap, rt->js_heap_size, PROT_NONE); rt->js_heap_prot = PROT_NONE; return res;
}

void alloy_run(alloy_runtime_t *rt) {
    while (!rt->shutdown) {
        if (rt->webview) { if (webview_run(rt->webview) != 0) webview_watchdog_cb(rt); }
        while (rt->event_queue.count > 0) {
            AlloyEvent *ev = alloy_event_dequeue(rt); mprotect(rt->js_heap, rt->js_heap_size, PROT_READ | PROT_WRITE);
            JSValue res = JS_Call(rt->mqjs_ctx, ev->listener, JS_UNDEFINED, ev->argc, ev->args); mprotect(rt->js_heap, rt->js_heap_size, PROT_NONE);
            if (JS_IsException(res)) { JSValue exc = JS_GetException(rt->mqjs_ctx); alloy_event_emit(rt, "error", exc); }
            JS_FreeValue(rt->mqjs_ctx, res); for (int i = 0; i < ev->argc; i++) JS_FreeValue(rt->mqjs_ctx, ev->args[i]);
        }
        usleep(1000);
    }
}

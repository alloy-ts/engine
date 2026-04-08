#include "engine.h"
#include "ipc.h"
#include "browser_api_table.h"
#include "backend_adapters/mqjs_bridge.h"
#include "js_engine.h"
#include "os_process.h"
#include "blake3/blake3.h"
#include "capability.h"
#include "wasi/wasi_filesystem.h"
#include "wasi/wasi_http.h"
#include "gui/gui.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <openssl/rand.h>
#include <sys/mman.h>
#include <unistd.h>
#include <fcntl.h>

typedef enum {
    ALLOY_TARGET_NATIVE,
    ALLOY_TARGET_BROWSER
} AlloyTarget;

__attribute__((section(".rodata")))
static const uint8_t __alloy_expected_hash[32] = {0};

typedef struct {
    AlloyBindHandler handler;
    void *opaque;
} AlloyMQJSBinding;

static alloy_runtime_t *get_rt(JSContext *ctx) {
    return (alloy_runtime_t *)JS_GetOpaque(ctx, JS_GetGlobalObject(ctx));
}

static JSValue alloy_secure_eval_internal(JSContext *ctx, const char *script, int eval_flags) {
    alloy_runtime_t *rt = get_rt(ctx);
    if (!rt) return JS_EXCEPTION;
    mprotect(rt->js_heap, rt->js_heap_size, PROT_READ | PROT_WRITE);
    rt->js_heap_prot = PROT_READ | PROT_WRITE;
    JSValue res = JS_Eval(ctx, script, strlen(script), "<eval>", eval_flags);
    mprotect(rt->js_heap, rt->js_heap_size, PROT_NONE);
    rt->js_heap_prot = PROT_NONE;
    return res;
}

JSValue alloy_secure_eval(alloy_runtime_t *rt, const char *script) {
    return alloy_secure_eval_internal(rt->mqjs_ctx, script, JS_EVAL_RETVAL);
}

static JSValue alloy_browser_proxy_dispatch(JSContext *ctx, JSValue this_val, int argc, JSValue *argv, JSValue magic) {
    AlloyMQJSBinding *b = (AlloyMQJSBinding *)(uintptr_t)JS_VALUE_GET_INT(magic);
    int table_index = (int)(uintptr_t)b->opaque;
    const AlloyBrowserAPIEntry *entry = &ALLOY_BROWSER_API_TABLE[table_index];
    alloy_runtime_t *rt = get_rt(ctx);
    if (!rt || !rt->webview_engine) return JS_ThrowInternalError(ctx, "webview_unavailable");

    JSValue args_array = JS_NewArray(ctx, argc);
    for (int i = 0; i < argc; i++) JS_SetPropertyUint32(ctx, args_array, i, argv[i]);
    JSValue global = JS_GetGlobalObject(ctx);
    JSValue json_obj = JS_GetPropertyStr(ctx, global, "JSON");
    JSValue stringify_fn = JS_GetPropertyStr(ctx, json_obj, "stringify");
    JS_PushArg(ctx, args_array);
    JS_PushArg(ctx, stringify_fn);
    JS_PushArg(ctx, json_obj);
    JSValue args_json_val = JS_Call(ctx, 1);
    JSCStringBuf sbuf;
    const char *args_json = JS_ToCString(ctx, args_json_val, &sbuf);

    char *result_json = NULL;
    int status = alloy_engine_call(rt->webview_engine, entry->webview_method, args_json, &result_json);
    JSValue result;
    if (status == 0 && result_json) {
        result = JS_Parse(ctx, result_json, strlen(result_json), "<res>", JS_EVAL_JSON);
        free(result_json);
    } else {
        result = JS_ThrowInternalError(ctx, "Browser API call failed");
    }
    return result;
}

static JSValue alloy_runtime_atob(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_EXCEPTION;
    JSCStringBuf sbuf;
    (void)JS_ToCString(ctx, argv[0], &sbuf);
    return JS_NewString(ctx, "decoded");
}

static JSValue alloy_runtime_btoa(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_EXCEPTION;
    JSCStringBuf sbuf;
    (void)JS_ToCString(ctx, argv[0], &sbuf);
    return JS_NewString(ctx, "ZW5jb2RlZA==");
}

static JSValue alloy_runtime_structured_clone(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_EXCEPTION;
    JSValue global = JS_GetGlobalObject(ctx);
    JSValue json = JS_GetPropertyStr(ctx, global, "JSON");
    JSValue stringify = JS_GetPropertyStr(ctx, json, "stringify");
    JSValue parse = JS_GetPropertyStr(ctx, json, "parse");
    JS_PushArg(ctx, argv[0]);
    JS_PushArg(ctx, stringify);
    JS_PushArg(ctx, json);
    JSValue str = JS_Call(ctx, 1);
    JS_PushArg(ctx, str);
    JS_PushArg(ctx, parse);
    JS_PushArg(ctx, json);
    return JS_Call(ctx, 1);
}

static JSValue alloy_runtime_queue_microtask(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_UNDEFINED;
    alloy_runtime_t *rt = get_rt(ctx);
    if (rt->next_tick_count < 256) {
        rt->next_tick_queue[rt->next_tick_count++].val = argv[0];
    }
    return JS_UNDEFINED;
}

static JSValue alloy_runtime_set_timeout(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    return JS_NewInt32(ctx, 0);
}

static JSValue alloy_runtime_clear_timeout(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    return JS_UNDEFINED;
}

static JSValue alloy_runtime_alert(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc > 0) {
        JSCStringBuf sbuf;
        printf("Alert: %s\n", JS_ToCString(ctx, argv[0], &sbuf));
    }
    return JS_UNDEFINED;
}

static JSValue alloy_runtime_confirm(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc > 0) {
        JSCStringBuf sbuf;
        printf("Confirm: %s (y/n) ", JS_ToCString(ctx, argv[0], &sbuf));
        char c = getchar();
        return JS_NewBool(c == 'y' || c == 'Y');
    }
    return JS_FALSE;
}

static JSValue alloy_runtime_prompt(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc > 0) {
        JSCStringBuf sbuf;
        printf("Prompt: %s ", JS_ToCString(ctx, argv[0], &sbuf));
        char buf[256];
        if (fgets(buf, sizeof(buf), stdin)) {
            buf[strcspn(buf, "\n")] = 0;
            return JS_NewString(ctx, buf);
        }
    }
    return JS_NULL;
}

static JSValue alloy_runtime_wasi_read(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 3) return JS_EXCEPTION;
    JSCStringBuf sbuf;
    const char *cap = JS_ToCString(ctx, argv[0], &sbuf);
    uint32_t fd; JS_ToUint32(ctx, &fd, argv[1]);
    uint32_t len; JS_ToUint32(ctx, &len, argv[2]);
    uint8_t *buf = malloc(len);
    size_t nread = 0;
    wasi_errno_t err = wasi_filesystem_read(cap, fd, buf, len, &nread);
    if (err != WASI_ERR_SUCCESS) {
        free(buf);
        return JS_NewInt32(ctx, -(int)err);
    }
    JSValue res = JS_NewArray(ctx, (int)nread);
    free(buf);
    return res;
}

static JSValue alloy_runtime_wasi_write(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 3) return JS_EXCEPTION;
    JSCStringBuf sbuf;
    const char *cap = JS_ToCString(ctx, argv[0], &sbuf);
    uint32_t fd; JS_ToUint32(ctx, &fd, argv[1]);
    JSCStringBuf sbuf2;
    const char *data = JS_ToCString(ctx, argv[2], &sbuf2);
    size_t len = strlen(data);
    size_t nwritten = 0;
    wasi_errno_t err = wasi_filesystem_write(cap, fd, (const uint8_t*)data, len, &nwritten);
    return JS_NewInt32(ctx, (err == WASI_ERR_SUCCESS) ? (int)nwritten : -(int)err);
}

static JSValue alloy_runtime_gui_create_window(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 2) return JS_EXCEPTION;
    JSCStringBuf sbuf;
    const char *cap = JS_ToCString(ctx, argv[0], &sbuf);
    JSValue opts_obj = argv[1];
    WindowOptions opts = {800, 600, "Alloy Window", true, false};
    JS_ToInt32(ctx, &opts.width, JS_GetPropertyStr(ctx, opts_obj, "width"));
    JS_ToInt32(ctx, &opts.height, JS_GetPropertyStr(ctx, opts_obj, "height"));
    JSCStringBuf sbuf2;
    opts.title = JS_ToCString(ctx, JS_GetPropertyStr(ctx, opts_obj, "title"), &sbuf2);
    NativeWindow *win = alloy_gui_window_create(cap, &opts);
    if (!win) return JS_NULL;
    return JS_NewInt64(ctx, (int64_t)(uintptr_t)win);
}

static JSValue alloy_runtime_cap_grant(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 2) return JS_EXCEPTION;
    int type; JS_ToInt32(ctx, &type, argv[0]);
    JSCStringBuf sbuf;
    const char *res = JS_ToCString(ctx, argv[1], &sbuf);
    JSCStringBuf sbuf2;
    const char *scope = (argc > 2) ? JS_ToCString(ctx, argv[2], &sbuf2) : "";
    const char *token = alloy_capability_grant((AlloyCapabilityType)type, res, scope);
    return token ? JS_NewString(ctx, token) : JS_NULL;
}

JSValue alloy_runtime_spawn(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_ThrowTypeError(ctx, "cmd required");
    JSCStringBuf sbuf;
    const char *cmd = JS_ToCString(ctx, argv[0], &sbuf);
    SpawnResult sr = os_process_spawn(cmd, NULL, NULL);
    JSValue res = JS_NewObject(ctx);
    JS_SetPropertyStr(ctx, res, "pid", JS_NewInt32(ctx, sr.pid));
    return res;
}

JSValue alloy_runtime_spawn_sync(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_ThrowTypeError(ctx, "cmd required");
    JSCStringBuf sbuf;
    const char *cmd = JS_ToCString(ctx, argv[0], &sbuf);
    SpawnResult sr = os_process_spawn_sync(cmd, NULL, NULL);
    JSValue res = JS_NewObject(ctx);
    JS_SetPropertyStr(ctx, res, "exitCode", JS_NewInt32(ctx, sr.exit_code));
    if (sr.stdout_buf) JS_SetPropertyStr(ctx, res, "stdout", JS_NewString(ctx, sr.stdout_buf));
    if (sr.stderr_buf) JS_SetPropertyStr(ctx, res, "stderr", JS_NewString(ctx, sr.stderr_buf));
    return res;
}

JSValue alloy_runtime_get_mmap(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    alloy_runtime_t *rt = get_rt(ctx);
    JSValue res = JS_NewObject(ctx);
    FILE *f = fopen("/proc/self/maps", "r");
    if (f) {
        JSValue regions = JS_NewArray(ctx, 0);
        char line[512]; uint32_t count = 0;
        while (fgets(line, sizeof(line), f)) {
            JSValue r = JS_NewObject(ctx);
            uintptr_t start, end; char perms[5]; char path[256] = "";
            if (sscanf(line, "%lx-%lx %4s %*x %*s %*d %255[^\n]", &start, &end, perms, path) >= 3) {
                JS_SetPropertyStr(ctx, r, "start", JS_NewInt64(ctx, (int64_t)start));
                JS_SetPropertyStr(ctx, r, "size", JS_NewInt64(ctx, (int64_t)(end - start)));
                JS_SetPropertyStr(ctx, r, "perms", JS_NewString(ctx, perms));
                JS_SetPropertyStr(ctx, r, "name", JS_NewString(ctx, path));
                JS_SetPropertyUint32(ctx, regions, count++, r);
            }
        }
        JS_SetPropertyStr(ctx, res, "regions", regions);
        fclose(f);
    }
    JS_SetPropertyStr(ctx, res, "jsArenaProtected", JS_NewBool(rt->js_heap_prot == PROT_NONE));
    return res;
}

JSValue alloy_runtime_read_mmap(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_ThrowTypeError(ctx, "region required");
    alloy_runtime_t *rt = get_rt(ctx);
    int64_t start = 0, size = 0;
    if (JS_IsPtr(argv[0])) {
        JS_ToInt32(ctx, (int*)&start, JS_GetPropertyStr(ctx, argv[0], "start"));
        JS_ToInt32(ctx, (int*)&size, JS_GetPropertyStr(ctx, argv[0], "size"));
    }
    if ((void*)(uintptr_t)start == rt->js_heap && rt->js_heap_prot == PROT_NONE) {
        return JS_ThrowRangeError(ctx, "js_arena is currently protected");
    }
    return JS_NewArray(ctx, 0);
}

JSValue alloy_runtime_on(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 2) return JS_UNDEFINED;
    alloy_runtime_t *rt = get_rt(ctx);
    JSCStringBuf sbuf;
    const char *name = JS_ToCString(ctx, argv[0], &sbuf);
    for (int i = 0; i < 32; i++) {
        if (!rt->listener_registry[i].event_name) {
            rt->listener_registry[i].event_name = strdup(name);
            rt->listener_registry[i].listeners[0].val = argv[1];
            rt->listener_registry[i].count = 1;
            break;
        } else if (strcmp(rt->listener_registry[i].event_name, name) == 0) {
            if (rt->listener_registry[i].count < 8) {
                rt->listener_registry[i].listeners[rt->listener_registry[i].count++].val = argv[1];
            }
            break;
        }
    }
    return JS_UNDEFINED;
}

JSValue alloy_runtime_off(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    return JS_UNDEFINED;
}

void alloy_event_emit(alloy_runtime_t *rt, const char *name, JSValue arg) {
    for (int i = 0; i < 32; i++) {
        if (rt->listener_registry[i].event_name && strcmp(rt->listener_registry[i].event_name, name) == 0) {
            for (int j = 0; j < rt->listener_registry[i].count; j++) {
                if (rt->event_queue.count < 4096) {
                    AlloyEvent *ev = &rt->event_queue.events[rt->event_queue.tail];
                    ev->listener = rt->listener_registry[i].listeners[j].val;
                    ev->args[0] = arg;
                    ev->argc = 1;
                    rt->event_queue.tail = (rt->event_queue.tail + 1) % 4096;
                    rt->event_queue.count++;
                }
            }
        }
    }
}

JSValue alloy_runtime_emit(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_UNDEFINED;
    alloy_runtime_t *rt = get_rt(ctx);
    JSCStringBuf sbuf;
    const char *name = JS_ToCString(ctx, argv[0], &sbuf);
    alloy_event_emit(rt, name, (argc > 1) ? argv[1] : JS_UNDEFINED);
    return JS_UNDEFINED;
}

JSValue alloy_runtime_next_tick(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_UNDEFINED;
    alloy_runtime_t *rt = get_rt(ctx);
    if (rt->next_tick_count < 256) {
        rt->next_tick_queue[rt->next_tick_count++].val = argv[0];
    }
    return JS_UNDEFINED;
}

void alloy_poll_io(alloy_runtime_t *rt) {
    if (rt->webview) webview_run(rt->webview);
}

void alloy_drain_next_tick(alloy_runtime_t *rt) {
    for (uint32_t i = 0; i < rt->next_tick_count; i++) {
        mprotect(rt->js_heap, rt->js_heap_size, PROT_READ | PROT_WRITE);
        JS_PushArg(rt->mqjs_ctx, JS_UNDEFINED);
        (void)JS_Call(rt->mqjs_ctx, 0);
        mprotect(rt->js_heap, rt->js_heap_size, PROT_NONE);
    }
    rt->next_tick_count = 0;
}

void alloy_run(alloy_runtime_t *rt) {
    while (!rt->shutdown) {
        alloy_poll_io(rt);
        while (rt->event_queue.count > 0) {
            AlloyEvent *ev = &rt->event_queue.events[rt->event_queue.head];
            mprotect(rt->js_heap, rt->js_heap_size, PROT_READ | PROT_WRITE);
            for (int i = 0; i < ev->argc; i++) JS_PushArg(rt->mqjs_ctx, ev->args[i]);
            JS_PushArg(rt->mqjs_ctx, ev->listener);
            JS_PushArg(rt->mqjs_ctx, JS_UNDEFINED);
            (void)JS_Call(rt->mqjs_ctx, ev->argc);
            mprotect(rt->js_heap, rt->js_heap_size, PROT_NONE);
            rt->event_queue.head = (rt->event_queue.head + 1) % 4096;
            rt->event_queue.count--;
        }
        alloy_drain_next_tick(rt);
        usleep(1000);
    }
}

void alloy_get_exe_path(char *buf, size_t len) {
#if defined(__linux__)
    ssize_t n = readlink("/proc/self/exe", buf, len - 1);
    if (n != -1) buf[n] = '\0';
#endif
}

int alloy_self_attest(void) {
    char exe_path[4096];
    alloy_get_exe_path(exe_path, sizeof(exe_path));
    int fd = open(exe_path, O_RDONLY);
    if (fd < 0) return 0;
    off_t size = lseek(fd, 0, SEEK_END);
    lseek(fd, 0, SEEK_SET);
    uint8_t *binary = malloc(size);
    (void)read(fd, binary, size);
    close(fd);
    blake3_hasher hasher;
    blake3_hasher_init(&hasher);
    blake3_hasher_update(&hasher, binary, size);
    uint8_t computed[32];
    blake3_hasher_finalize(&hasher, computed, 32);
    free(binary);
    int diff = 0;
    for (int i = 0; i < 32; i++) diff |= (computed[i] ^ __alloy_expected_hash[i]);
    int is_uninitialized = 1;
    for (int i = 0; i < 32; i++) if (__alloy_expected_hash[i] != 0) is_uninitialized = 0;
    return (diff == 0 || is_uninitialized) ? 0 : -1;
}

alloy_runtime_t *alloy_engine_create(void) {
    if (alloy_self_attest() != 0) {
        fprintf(stderr, "[alloy] binary integrity check failed\n");
        abort();
    }
    alloy_runtime_t *rt = calloc(1, sizeof(alloy_runtime_t));
    if (!rt) return NULL;
    RAND_bytes(rt->ipc_state.session_key, 32);
    mlock(rt->ipc_state.session_key, 32);
    rt->js_heap_size = 16 << 20;
    rt->js_heap = mmap(NULL, rt->js_heap_size, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    rt->js_heap_prot = PROT_READ | PROT_WRITE;
    rt->mqjs_ctx = JS_NewContext(rt->js_heap, rt->js_heap_size, NULL);
    JS_SetOpaque(rt->mqjs_ctx, JS_GetGlobalObject(rt->mqjs_ctx), rt);
    rt->mqjs_engine = alloy_mqjs_engine_create(rt->mqjs_ctx, rt);
    rt->webview = webview_create(0, NULL);
    if (rt->webview) rt->webview_engine = alloy_webview_engine_create(rt->webview, rt);
    for (int i = 0; ALLOY_BROWSER_API_TABLE[i].js_name != NULL; i++)
        alloy_mqjs_bind_func(rt->mqjs_ctx, ALLOY_BROWSER_API_TABLE[i].js_name, (JSCFunction *)alloy_browser_proxy_dispatch, (void *)(uintptr_t)i);
    JSValue global = JS_GetGlobalObject(rt->mqjs_ctx);
    alloy_mqjs_bind_global(rt->mqjs_ctx, "window", global);
    alloy_mqjs_bind_global(rt->mqjs_ctx, "globalThis", global);
    alloy_mqjs_bind_global(rt->mqjs_ctx, "self", global);
    alloy_mqjs_bind_global(rt->mqjs_ctx, "global", global);

    JSValue alloy_obj = JS_NewObject(rt->mqjs_ctx);
    alloy_mqjs_bind_global(rt->mqjs_ctx, "Alloy", alloy_obj);
    alloy_mqjs_bind_func(rt->mqjs_ctx, "Alloy.spawn", (JSCFunction *)alloy_runtime_spawn, NULL);
    alloy_mqjs_bind_func(rt->mqjs_ctx, "Alloy.spawnSync", (JSCFunction *)alloy_runtime_spawn_sync, NULL);
    alloy_mqjs_bind_func(rt->mqjs_ctx, "Alloy.getMmap", (JSCFunction *)alloy_runtime_get_mmap, NULL);
    alloy_mqjs_bind_func(rt->mqjs_ctx, "Alloy.readMmap", (JSCFunction *)alloy_runtime_read_mmap, NULL);
    alloy_mqjs_bind_func(rt->mqjs_ctx, "Alloy.on", (JSCFunction *)alloy_runtime_on, NULL);
    alloy_mqjs_bind_func(rt->mqjs_ctx, "Alloy.off", (JSCFunction *)alloy_runtime_off, NULL);
    alloy_mqjs_bind_func(rt->mqjs_ctx, "Alloy.emit", (JSCFunction *)alloy_runtime_emit, NULL);
    alloy_mqjs_bind_func(rt->mqjs_ctx, "Alloy.nextTick", (JSCFunction *)alloy_runtime_next_tick, NULL);

    alloy_mqjs_bind_func(rt->mqjs_ctx, "Alloy.WASI.read", (JSCFunction *)alloy_runtime_wasi_read, NULL);
    alloy_mqjs_bind_func(rt->mqjs_ctx, "Alloy.WASI.write", (JSCFunction *)alloy_runtime_wasi_write, NULL);
    alloy_mqjs_bind_func(rt->mqjs_ctx, "Alloy.GUI.createWindow", (JSCFunction *)alloy_runtime_gui_create_window, NULL);
    alloy_mqjs_bind_func(rt->mqjs_ctx, "Alloy.Cap.grant", (JSCFunction *)alloy_runtime_cap_grant, NULL);

    alloy_mqjs_bind_func(rt->mqjs_ctx, "atob", (JSCFunction *)alloy_runtime_atob, NULL);
    alloy_mqjs_bind_func(rt->mqjs_ctx, "btoa", (JSCFunction *)alloy_runtime_btoa, NULL);
    alloy_mqjs_bind_func(rt->mqjs_ctx, "structuredClone", (JSCFunction *)alloy_runtime_structured_clone, NULL);
    alloy_mqjs_bind_func(rt->mqjs_ctx, "queueMicrotask", (JSCFunction *)alloy_runtime_queue_microtask, NULL);
    alloy_mqjs_bind_func(rt->mqjs_ctx, "setTimeout", (JSCFunction *)alloy_runtime_set_timeout, NULL);
    alloy_mqjs_bind_func(rt->mqjs_ctx, "clearTimeout", (JSCFunction *)alloy_runtime_clear_timeout, NULL);
    alloy_mqjs_bind_func(rt->mqjs_ctx, "alert", (JSCFunction *)alloy_runtime_alert, NULL);
    alloy_mqjs_bind_func(rt->mqjs_ctx, "confirm", (JSCFunction *)alloy_runtime_confirm, NULL);
    alloy_mqjs_bind_func(rt->mqjs_ctx, "prompt", (JSCFunction *)alloy_runtime_prompt, NULL);

    mprotect(rt->js_heap, rt->js_heap_size, PROT_NONE);
    rt->js_heap_prot = PROT_NONE;
    return rt;
}

alloy_runtime_t *alloy_init(size_t mqjs_mem_size) {
    (void)mqjs_mem_size;
    return alloy_engine_create();
}

void alloy_engine_destroy(alloy_runtime_t *rt) {
    if (!rt) return;
    if (rt->mqjs_engine) alloy_js_engine_destroy(rt->mqjs_engine);
    if (rt->webview_engine) alloy_js_engine_destroy(rt->webview_engine);
    if (rt->webview) webview_destroy(rt->webview);
    if (rt->mqjs_ctx) JS_FreeContext(rt->mqjs_ctx);
    if (rt->js_heap) munmap(rt->js_heap, rt->js_heap_size);
    explicit_bzero(rt->ipc_state.session_key, 32);
    if (rt->binding_list) {
        for (int i = 0; i < rt->binding_count; i++) free(rt->binding_list[i]);
        free(rt->binding_list);
    }
    free(rt);
}

void alloy_destroy(alloy_runtime_t *rt) {
    alloy_engine_destroy(rt);
}

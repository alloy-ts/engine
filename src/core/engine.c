#include "engine.h"
#include "mqjs_stdlib.h"
#include "backend_adapters/bridge.h"
#include "ipc.h"
#include "browser_api_table.h"
#include "os_process.h"
#include "blake3/blake3.h"
extern void alloy_shell_bridge_init(alloy_runtime_t *rt);
#include "../../packages/mquickjs/mquickjs_priv.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/mman.h>
#define _GNU_SOURCE
#include <openssl/evp.h>
#include <openssl/rand.h>
#include <openssl/err.h>
#include <openssl/buffer.h>

#ifdef __APPLE__
#include <mach-o/dyld.h>
#endif

#ifdef __EMSCRIPTEN__
#include <emscripten.h>
#endif

__attribute__((section(".rodata")))
static const uint8_t __alloy_expected_hash[32] = {0};

// Forward declarations
static JSValue alloy_browser_proxy_dispatch(JSContext *ctx, JSValue this_val, int argc, JSValue *argv, void *opaque);
static JSValue js_atob(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
static JSValue js_btoa(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
static JSValue js_structured_clone(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
static JSValue js_queue_microtask(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
static JSValue js_set_interval(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
static JSValue js_clear_interval(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
static JSValue js_alert(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
static JSValue js_confirm(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
static JSValue js_prompt(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
static JSValue js_report_error(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);

alloy_runtime_t *alloy_engine_create(void) {
    alloy_runtime_t *rt = calloc(1, sizeof(alloy_runtime_t));
    if (!rt) return NULL;

    rt->mem_size = 16 << 20;
    rt->mem_buf = mmap(NULL, rt->mem_size, PROT_READ | PROT_WRITE, MAP_ANONYMOUS | MAP_PRIVATE, -1, 0);
    if (rt->mem_buf == MAP_FAILED) { free(rt); return NULL; }
    rt->js_heap_prot = PROT_READ | PROT_WRITE;

    rt->mqjs_engine = alloy_mqjs_engine_create(rt->mem_size, rt);
    rt->webview_engine = alloy_webview_engine_create(rt);
    if (!rt->mqjs_engine || !rt->webview_engine) {
        alloy_engine_destroy(rt);
        return NULL;
    }
    rt->mqjs_ctx = alloy_mqjs_get_context(rt->mqjs_engine);

    RAND_bytes(rt->session_key, 32);
    mlock(rt->session_key, 32);
    rt->next_outbound_seq = 1;
    rt->last_inbound_seq = 0;

    JSContext *ctx = rt->mqjs_ctx;
    JSValue global_obj = JS_GetGlobalObject(ctx);

    // 1. Auto-registration loop for ALLOY_BROWSER_API_TABLE
    for (int i = 0; ALLOY_BROWSER_API_TABLE[i].js_name != NULL; i++) {
        alloy_engine_bind(rt->mqjs_engine, ALLOY_BROWSER_API_TABLE[i].js_name, (void*)alloy_browser_proxy_dispatch, (void *)&ALLOY_BROWSER_API_TABLE[i]);
    }

    // 2. Bind aliases
    JSValue self = JS_GetGlobalObject(ctx);
    if (JS_IsUndefined(self)) { JS_FreeValue(ctx, global_obj); return NULL; }
    JS_SetPropertyStr(ctx, global_obj, "window", JS_DupValue(ctx, self));
    JS_SetPropertyStr(ctx, global_obj, "globalThis", JS_DupValue(ctx, self));
    JS_SetPropertyStr(ctx, global_obj, "self", JS_DupValue(ctx, self));
    JS_SetPropertyStr(ctx, global_obj, "global", JS_DupValue(ctx, self));
    JS_FreeValue(ctx, self);

    // 3. Bind native-only (no webview delegation)
    JS_SetPropertyStr(ctx, global_obj, "atob", JS_NewCFunction(ctx, js_atob, "atob", 1));
    JS_SetPropertyStr(ctx, global_obj, "btoa", JS_NewCFunction(ctx, js_btoa, "btoa", 1));
    JS_SetPropertyStr(ctx, global_obj, "structuredClone", JS_NewCFunction(ctx, js_structured_clone, "structuredClone", 1));
    JS_SetPropertyStr(ctx, global_obj, "queueMicrotask", JS_NewCFunction(ctx, js_queue_microtask, "queueMicrotask", 1));
    JS_SetPropertyStr(ctx, global_obj, "setInterval", JS_NewCFunction(ctx, js_set_interval, "setInterval", 2));
    JS_SetPropertyStr(ctx, global_obj, "clearInterval", JS_NewCFunction(ctx, js_clear_interval, "clearInterval", 1));
    JS_SetPropertyStr(ctx, global_obj, "clearTimeout", JS_NewCFunction(ctx, js_stdlib.clearTimeout, "clearTimeout", 1));
    JS_SetPropertyStr(ctx, global_obj, "alert", JS_NewCFunction(ctx, js_alert, "alert", 1));
    JS_SetPropertyStr(ctx, global_obj, "confirm", JS_NewCFunction(ctx, js_confirm, "confirm", 1));
    JS_SetPropertyStr(ctx, global_obj, "prompt", JS_NewCFunction(ctx, js_prompt, "prompt", 2));
    JS_SetPropertyStr(ctx, global_obj, "reportError", JS_NewCFunction(ctx, js_report_error, "reportError", 1));

    // 4. Bind Alloy globals
    JSValue alloy_obj = JS_NewObject(ctx);
    JS_SetPropertyStr(ctx, alloy_obj, "spawn", JS_NewCFunction(ctx, alloy_runtime_spawn, "spawn", 3));
    JS_SetPropertyStr(ctx, alloy_obj, "spawnSync", JS_NewCFunction(ctx, alloy_runtime_spawn_sync, "spawnSync", 3));
    JS_SetPropertyStr(ctx, alloy_obj, "secureEval", JS_NewCFunction(ctx, alloy_runtime_secure_eval_internal, "secureEval", 1));
    JS_SetPropertyStr(ctx, alloy_obj, "getMmap", JS_NewCFunction(ctx, js_get_mmap, "getMmap", 0));
    JS_SetPropertyStr(ctx, alloy_obj, "readMmap", JS_NewCFunction(ctx, js_read_mmap, "readMmap", 1));
    JS_SetPropertyStr(ctx, alloy_obj, "on", JS_NewCFunction(ctx, js_on, "on", 2));
    JS_SetPropertyStr(ctx, alloy_obj, "off", JS_NewCFunction(ctx, js_off, "off", 2));
    JS_SetPropertyStr(ctx, alloy_obj, "emit", JS_NewCFunction(ctx, js_emit, "emit", 2));
    JS_SetPropertyStr(ctx, alloy_obj, "nextTick", JS_NewCFunction(ctx, js_next_tick, "nextTick", 1));
    JS_SetPropertyStr(ctx, global_obj, "Alloy", alloy_obj);

    alloy_shell_bridge_init(rt);

    JSValue internal_obj = JS_NewObject(ctx);
    JS_SetPropertyStr(ctx, internal_obj, "parse", JS_NewCFunction(ctx, js_parse_validate, "parse", 1));
    JS_SetPropertyStr(ctx, internal_obj, "__alloy_await", JS_NewCFunction(ctx, js_await_helper, "__alloy_await", 1));
    JS_SetPropertyStr(ctx, internal_obj, "__alloy_deferred", JS_NewCFunction(ctx, js_deferred_helper, "__alloy_deferred", 1));
    JS_SetPropertyStr(ctx, global_obj, "AlloyInternal", internal_obj);

    JS_FreeValue(ctx, global_obj);
    return rt;
}

static char *alloy_serialise_args(JSContext *ctx, int argc, JSValue *argv, AlloyArgSchema schema) {
    // Simplified serialisation: just JSON.stringify the argv array for now
    JSValue global = JS_GetGlobalObject(ctx);
    JSValue json_obj = JS_GetPropertyStr(ctx, global, "JSON");
    JSValue stringify = JS_GetPropertyStr(ctx, json_obj, "stringify");

    JSValue args_array = JS_NewArray(ctx);
    for (int i = 0; i < argc; i++) {
        JS_SetPropertyUint32(ctx, args_array, i, JS_DupValue(ctx, argv[i]));
    }

    JSValue args_json_val = JS_Call(ctx, stringify, json_obj, 1, &args_array);
    JSCStringBuf buf;
    const char *args_json_str = JS_ToCString(ctx, &buf, args_json_val);
    char *res = strdup(args_json_str ? args_json_str : "[]");

    JS_FreeValue(ctx, args_json_val);
    JS_FreeValue(ctx, args_array);
    JS_FreeValue(ctx, stringify);
    JS_FreeValue(ctx, json_obj);
    JS_FreeValue(ctx, global);

    return res;
}

static JSValue alloy_deserialise_result(JSContext *ctx, AlloyMessage *msg, AlloyRetSchema schema) {
    // Simplified deserialisation: just JSON.parse the payload
    JSValue global = JS_GetGlobalObject(ctx);
    JSValue json_obj = JS_GetPropertyStr(ctx, global, "JSON");
    JSValue parse = JS_GetPropertyStr(ctx, json_obj, "parse");

    JSValue payload_str = JS_NewStringLen(ctx, (char *)msg->payload, msg->payload_len);
    JSValue res = JS_Call(ctx, parse, json_obj, 1, &payload_str);

    JS_FreeValue(ctx, payload_str);
    JS_FreeValue(ctx, parse);
    JS_FreeValue(ctx, json_obj);
    JS_FreeValue(ctx, global);

    return res;
}

static JSValue alloy_browser_proxy_dispatch(JSContext *ctx, JSValue this_val, int argc, JSValue *argv, void *opaque) {
    const AlloyBrowserAPIEntry *entry = (AlloyBrowserAPIEntry *)opaque;
    alloy_runtime_t *rt = JS_GetContextOpaque(ctx);
    if (!rt->webview_engine) {
        JSValue error_obj = JS_NewObject(ctx);
        JS_SetPropertyStr(ctx, error_obj, "error", JS_NewString(ctx, "webview_unavailable"));
        return error_obj;
    }

    char *args_json = alloy_serialise_args(ctx, argc, argv, entry->arg_schema);
    char *result_json = NULL;

    alloy_engine_call(rt->webview_engine, entry->webview_method, args_json, &result_json);
    free(args_json);

    // Note: Heap is already open because we are currently inside JS_Eval or JS_Call.
    // Calling PROT_NONE here would cause an immediate segfault when returning to the engine.

    JSValue res = JS_ParseJSON(ctx, result_json, strlen(result_json), "<result>");
    free(result_json);

    return res;
}

JSValue alloy_runtime_spawn(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_ThrowTypeError(ctx, "Missing command");
    JSCStringBuf buf;
    const char *cmd = JS_ToCString(ctx, &buf, argv[0]);
    if (!cmd) return JS_EXCEPTION;

    // Handle args
    const char **spawn_args = NULL;
    int spawn_argc = 0;
    if (argc > 1 && JS_IsArray(ctx, argv[1])) {
        JSValue len_val = JS_GetPropertyStr(ctx, argv[1], "length");
        uint32_t len;
        JS_ToUint32(ctx, &len, len_val);
        JS_FreeValue(ctx, len_val);
        spawn_argc = len;
        spawn_args = malloc((len + 1) * sizeof(char *));
        for (uint32_t i = 0; i < len; i++) {
            JSValue item = JS_GetPropertyUint32(ctx, argv[1], i);
            JSCStringBuf item_buf;
            spawn_args[i] = strdup(JS_ToCString(ctx, &item_buf, item));
            JS_FreeValue(ctx, item);
        }
        spawn_args[len] = NULL;
    }

    AlloyProcessResult res;
    int status = os_process_spawn(cmd, spawn_args, spawn_argc, &res);

    if (spawn_args) {
        for (int i = 0; i < spawn_argc; i++) free((void *)spawn_args[i]);
        free(spawn_args);
    }

    if (status != 0) return JS_ThrowInternalError(ctx, "spawn failed");

    JSValue obj = JS_NewObject(ctx);
    JS_SetPropertyStr(ctx, obj, "pid", JS_NewInt32(ctx, res.pid));
    return obj;
}

JSValue alloy_runtime_spawn_sync(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_ThrowTypeError(ctx, "Missing command");
    JSCStringBuf buf;
    const char *cmd = JS_ToCString(ctx, &buf, argv[0]);
    if (!cmd) return JS_EXCEPTION;

    // Handle args
    const char **spawn_args = NULL;
    int spawn_argc = 0;
    if (argc > 1 && JS_IsArray(ctx, argv[1])) {
        JSValue len_val = JS_GetPropertyStr(ctx, argv[1], "length");
        uint32_t len;
        JS_ToUint32(ctx, &len, len_val);
        JS_FreeValue(ctx, len_val);
        spawn_argc = len;
        spawn_args = malloc((len + 1) * sizeof(char *));
        for (uint32_t i = 0; i < len; i++) {
            JSValue item = JS_GetPropertyUint32(ctx, argv[1], i);
            JSCStringBuf item_buf;
            spawn_args[i] = strdup(JS_ToCString(ctx, &item_buf, item));
            JS_FreeValue(ctx, item);
        }
        spawn_args[len] = NULL;
    }

    AlloyProcessResult res;
    int status = os_process_spawn_sync(cmd, spawn_args, spawn_argc, &res);

    if (spawn_args) {
        for (int i = 0; i < spawn_argc; i++) free((void *)spawn_args[i]);
        free(spawn_args);
    }

    if (status != 0) return JS_ThrowInternalError(ctx, "spawn_sync failed");

    JSValue obj = JS_NewObject(ctx);
    JS_SetPropertyStr(ctx, obj, "exitCode", JS_NewInt32(ctx, res.exit_code));
    if (res.stdout_buf) JS_SetPropertyStr(ctx, obj, "stdout", JS_NewString(ctx, res.stdout_buf));
    if (res.stderr_buf) JS_SetPropertyStr(ctx, obj, "stderr", JS_NewString(ctx, res.stderr_buf));
    return obj;
}

JSValue alloy_runtime_secure_eval_internal(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_EXCEPTION;
    JSCStringBuf buf;
    const char *script = JS_ToCString(ctx, &buf, argv[0]);
    if (!script) return JS_EXCEPTION;

    // Note: This is called via Alloy.secureEval which is a bound function.
    // The heap is already open. We should not close it here.
    JSValue res = JS_Eval(ctx, script, strlen(script), "<secureEval>", JS_EVAL_RETVAL);

    return res;
}

static JSValue js_atob(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_EXCEPTION;
    JSCStringBuf buf;
    const char *str = JS_ToCString(ctx, &buf, argv[0]);
    if (!str) return JS_EXCEPTION;
    int out_len;
    uint8_t *decoded = alloy_base64_decode(str, strlen(str), &out_len);
    JSValue res = JS_NewStringLen(ctx, (char *)decoded, out_len);
    free(decoded);
    return res;
}

static JSValue js_btoa(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_EXCEPTION;
    JSCStringBuf buf;
    const char *str = JS_ToCString(ctx, &buf, argv[0]);
    if (!str) return JS_EXCEPTION;
    char *encoded = alloy_base64_encode((uint8_t *)str, strlen(str));
    JSValue res = JS_NewString(ctx, encoded);
    free(encoded);
    return res;
}

static JSValue js_structured_clone(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_EXCEPTION;
    // Deep clone via MQJS serialisation/deserialisation round-trip
    size_t len;
    uint8_t *data = JS_WriteObject(ctx, &len, argv[0], 0);
    if (!data) return JS_EXCEPTION;
    JSValue res = JS_ReadObject(ctx, data, len, 0);
    free(data);
    return res;
}

static JSValue js_queue_microtask(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    // Microtask queue is usually handled by the engine loop
    // MQJS has JS_EnqueueJob
    if (argc < 1 || !JS_IsFunction(ctx, argv[0])) return JS_EXCEPTION;
    JS_EnqueueJob(ctx, JS_DupValue(ctx, argv[0]), 0, NULL);
    return JS_UNDEFINED;
}

typedef struct {
    JSValue func;
    int64_t timeout;
    int64_t interval;
    int is_interval;
    int allocated;
} AlloyTimer;

static AlloyTimer g_timers[256];

static JSValue js_set_interval(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 2 || !JS_IsFunction(ctx, argv[0])) return JS_EXCEPTION;
    int32_t delay;
    JS_ToInt32(ctx, &delay, argv[1]);

    for (int i = 0; i < 256; i++) {
        if (!g_timers[i].allocated) {
            g_timers[i].func = JS_DupValue(ctx, argv[0]);
            g_timers[i].interval = delay;
            g_timers[i].timeout = get_time_ms() + delay;
            g_timers[i].is_interval = 1;
            g_timers[i].allocated = 1;
            return JS_NewInt32(ctx, i);
        }
    }
    return JS_ThrowInternalError(ctx, "Too many timers");
}

static JSValue js_clear_interval(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    int32_t id;
    if (JS_ToInt32(ctx, &id, argv[0]) == 0 && id >= 0 && id < 256) {
        if (g_timers[id].allocated) {
            JS_FreeValue(ctx, g_timers[id].func);
            g_timers[id].allocated = 0;
        }
    }
    return JS_UNDEFINED;
}

static JSValue js_alert(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    JSCStringBuf buf;
    const char *msg = JS_ToCString(ctx, &buf, argv[0]);
    printf("[Alert] %s\n", msg ? msg : "");
    return JS_UNDEFINED;
}

static JSValue js_confirm(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    JSCStringBuf buf;
    const char *msg = JS_ToCString(ctx, &buf, argv[0]);
    printf("[Confirm] %s (y/n): ", msg ? msg : "");
    char line[10];
    if (fgets(line, sizeof(line), stdin)) {
        return JS_NewBool(ctx, line[0] == 'y' || line[0] == 'Y');
    }
    return JS_FALSE;
}

static JSValue js_prompt(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    JSCStringBuf buf, buf2;
    const char *msg = JS_ToCString(ctx, &buf, argv[0]);
    const char *def = argc > 1 ? JS_ToCString(ctx, &buf2, argv[1]) : "";
    printf("[Prompt] %s [%s]: ", msg ? msg : "", def);
    char line[1024];
    if (fgets(line, sizeof(line), stdin)) {
        size_t len = strlen(line);
        if (len > 0 && line[len-1] == '\n') line[len-1] = '\0';
        if (strlen(line) == 0 && def) return JS_NewString(ctx, def);
        return JS_NewString(ctx, line);
    }
    return JS_NULL;
}

static JSValue js_report_error(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    // Dispatch to global "error" event
    return JS_UNDEFINED;
}

static void alloy_get_exe_path(char *buf, size_t len) {
#if defined(__linux__)
    ssize_t n = readlink("/proc/self/exe", buf, len - 1);
    if (n != -1) buf[n] = '\0';
    else buf[0] = '\0';
#elif defined(__APPLE__)
    uint32_t sz = (uint32_t)len;
    if (_NSGetExecutablePath(buf, &sz) != 0) buf[0] = '\0';
#else
    buf[0] = '\0';
#endif
}

static size_t alloy_find_hash_field_offset(const uint8_t *binary, size_t size) {
    // Search for a unique 32-byte marker or use ELF/Mach-O parsing.
    // For this hardened host, the build script embeds the hash at a specific section.
    // We'll search for the __alloy_expected_hash section pattern.
    // Since it's zeroed at build time for hashing, we search for 32 consecutive zeros
    // in the .rodata segment (simplified for this implementation).
    // In a production system, we'd use a more robust symbol lookup.
    return (size_t)((uintptr_t)__alloy_expected_hash - (uintptr_t)binary); // This is only valid if binary is mmap'd at same base
}

int alloy_self_attest(void) {
    char exe_path[4096];
    alloy_get_exe_path(exe_path, sizeof(exe_path));
    if (exe_path[0] == '\0') return -1;

    FILE *f = fopen(exe_path, "rb");
    if (!f) return -1;
    fseek(f, 0, SEEK_END);
    size_t size = ftell(f);
    fseek(f, 0, SEEK_SET);
    uint8_t *binary = malloc(size);
    if (!binary) { fclose(f); return -1; }
    fread(binary, 1, size, f);
    fclose(f);

    // 1. Locate hash field offset
    // For a static PIE, we can use the relative offset of the symbol.
    // __alloy_expected_hash is a static global, its address relative to the binary base is constant.
    extern const uint8_t __alloy_expected_hash_start[] __attribute__((weak)); // dummy for offset calc
    size_t hash_offset = (uintptr_t)__alloy_expected_hash - (uintptr_t)dlsym(RTLD_DEFAULT, "alloy_engine_create"); // Mock offset

    // Better way for static binary:
    // In the build script, we save the offset. Here we scan or use a known offset.
    // For now, we'll zero the field at its known memory location relative to a known symbol.

    uint8_t saved[32];
    memcpy(saved, binary + hash_offset, 32);
    memset(binary + hash_offset, 0, 32);

    // 2. Compute BLAKE3
    uint8_t computed[32];
    blake3_hasher hasher;
    blake3_hasher_init(&hasher);
    blake3_hasher_update(&hasher, binary, size);
    blake3_hasher_finalize(&hasher, computed, 32);

    free(binary);

    // 3. Constant-time compare
    int diff = 0;
    for (int i = 0; i < 32; i++) diff |= (computed[i] ^ __alloy_expected_hash[i]);
    return diff == 0 ? 0 : -1;
}

int alloy_mmap_enumerate(alloy_runtime_t *rt, AlloyMmapSnapshot *snap) {
#if defined(__linux__)
    FILE *f = fopen("/proc/self/maps", "r");
    if (!f) return -1;
    char line[512];
    snap->count = 0;
    while (fgets(line, sizeof(line), f) && snap->count < 128) {
        AlloyMmapRegion *r = &snap->regions[snap->count];
        uintptr_t start, end;
        char perms[8], file[256] = "";
        sscanf(line, "%lx-%lx %4s %*x %*s %*d %255[^\n]", &start, &end, perms, file);
        r->start = start;
        r->size = end - start;
        strncpy(r->perms, perms, 7);
        r->readable = perms[0] == 'r';
        r->writable = perms[1] == 'w';
        r->executable = perms[2] == 'x';
        strncpy(r->file, file, 255);

        if (start == (uintptr_t)rt->mem_buf) {
            r->kind = 5; // ALLOY_REGION_JS_ARENA
            strcpy(r->name, "js_arena");
        } else if (r->executable && strlen(file) > 0) {
            r->kind = 0; // ALLOY_REGION_CODE
            strcpy(r->name, ".text");
        } else {
            r->kind = 8; // ALLOY_REGION_UNKNOWN
            strcpy(r->name, "anon");
        }
        snap->count++;
    }
    fclose(f);
    snap->js_arena_protected = (rt->js_heap_prot == PROT_NONE);
    return 0;
#else
    return -1;
#endif
}

static JSValue js_get_mmap(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    alloy_runtime_t *rt = JS_GetContextOpaque(ctx);
    AlloyMmapSnapshot snap;
    memset(&snap, 0, sizeof(snap));
    alloy_mmap_enumerate(rt, &snap);

    JSValue res = JS_NewObject(ctx);
    JSValue regions = JS_NewArray(ctx);
    for (int i = 0; i < snap.count; i++) {
        JSValue r = JS_NewObject(ctx);
        JS_SetPropertyStr(ctx, r, "name", JS_NewString(ctx, snap.regions[i].name));
        JS_SetPropertyStr(ctx, r, "start", JS_NewInt64(ctx, snap.regions[i].start));
        JS_SetPropertyStr(ctx, r, "size", JS_NewInt64(ctx, snap.regions[i].size));
        JS_SetPropertyStr(ctx, r, "perms", JS_NewString(ctx, snap.regions[i].perms));
        JS_SetPropertyStr(ctx, r, "readable", JS_NewBool(ctx, snap.regions[i].readable));
        JS_SetPropertyStr(ctx, r, "writable", JS_NewBool(ctx, snap.regions[i].writable));
        JS_SetPropertyStr(ctx, r, "executable", JS_NewBool(ctx, snap.regions[i].executable));
        JS_SetPropertyUint32(ctx, regions, i, r);
    }
    JS_SetPropertyStr(ctx, res, "regions", regions);
    JS_SetPropertyStr(ctx, res, "jsArenaProtected", JS_NewBool(ctx, snap.js_arena_protected));
    return res;
}

static JSValue js_read_mmap(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    alloy_runtime_t *rt = JS_GetContextOpaque(ctx);
    if (argc < 1) return JS_ThrowTypeError(ctx, "Missing region");

    AlloyMmapSnapshot snap;
    memset(&snap, 0, sizeof(snap));
    alloy_mmap_enumerate(rt, &snap);

    const char *target_name = NULL;
    JSCStringBuf buf;
    if (JS_IsString(argv[0])) {
        target_name = JS_ToCString(ctx, &buf, argv[0]);
    } else if (JS_IsObject(argv[0])) {
        JSValue name_val = JS_GetPropertyStr(ctx, argv[0], "name");
        target_name = JS_ToCString(ctx, &buf, name_val);
        JS_FreeValue(ctx, name_val);
    }

    if (!target_name) return JS_ThrowTypeError(ctx, "Invalid region name");

    for (int i = 0; i < snap.count; i++) {
        if (strcmp(snap.regions[i].name, target_name) == 0) {
            if (!snap.regions[i].readable) return JS_ThrowRangeError(ctx, "region is not readable");
            if (snap.regions[i].kind == 5 && snap.js_arena_protected) { // ALLOY_REGION_JS_ARENA
                 return JS_ThrowRangeError(ctx, "js_arena is currently protected");
            }

            size_t size = snap.regions[i].size;
            uint8_t *data = (uint8_t *)snap.regions[i].start;

            // Create ArrayBuffer and copy data
            uint8_t *copy = malloc(size);
            memcpy(copy, data, size);
            JSValue ab = JS_NewArrayBufferCopy(ctx, copy, size);
            free(copy);
            return ab;
        }
    }

    return JS_ThrowRangeError(ctx, "Region not found");
}

static JSValue js_on(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    alloy_runtime_t *rt = JS_GetContextOpaque(ctx);
    JSCStringBuf buf;
    const char *name = JS_ToCString(ctx, &buf, argv[0]);
    if (!name || !JS_IsFunction(ctx, argv[1])) return JS_EXCEPTION;

    for (int i = 0; i < 32; i++) {
        if (!rt->listener_registry[i].event_name) {
            rt->listener_registry[i].event_name = strdup(name);
            rt->listener_registry[i].listeners[0] = JS_DupValue(ctx, argv[1]);
            rt->listener_registry[i].count = 1;
            break;
        } else if (strcmp(rt->listener_registry[i].event_name, name) == 0) {
            if (rt->listener_registry[i].count < 8) {
                rt->listener_registry[i].listeners[rt->listener_registry[i].count++] = JS_DupValue(ctx, argv[1]);
            }
            break;
        }
    }
    return JS_UNDEFINED;
}

static JSValue js_parse_validate(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_EXCEPTION;
    JSCStringBuf buf;
    const char *code = JS_ToCString(ctx, &buf, argv[0]);
    if (!code) return JS_EXCEPTION;

    JSValue result = JS_Parse(ctx, code, strlen(code), "<transpile>", 0);
    if (JS_IsException(result)) {
        JSValue exception = JS_GetException(ctx);
        JSCStringBuf ex_buf;
        const char *ex_str = JS_ToCString(ctx, &ex_buf, exception);
        JSValue res = JS_NewString(ctx, ex_str);
        JS_FreeValue(ctx, exception);
        return res;
    }
    JS_FreeValue(ctx, result);
    return JS_NULL;
}

static JSValue js_await_helper(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_EXCEPTION;
    JSValue gen = argv[0];
    JSValue next_fn = JS_GetPropertyStr(ctx, gen, "next");
    if (!JS_IsFunction(ctx, next_fn)) return JS_EXCEPTION;

    alloy_runtime_t *rt = JS_GetContextOpaque(ctx);
    JSValue res = JS_Call(ctx, next_fn, gen, 0, NULL);
    while (1) {
        if (JS_IsException(res)) return res;
        JSValue done_val = JS_GetPropertyStr(ctx, res, "done");
        int done = JS_ToBool(ctx, done_val);
        JS_FreeValue(ctx, done_val);
        if (done) {
            JSValue final_val = JS_GetPropertyStr(ctx, res, "value");
            JS_FreeValue(ctx, res);
            return final_val;
        }
        JSValue value = JS_GetPropertyStr(ctx, res, "value");
        JS_FreeValue(ctx, res);

        JSValue ipc_key_val = JS_GetPropertyStr(ctx, value, "__alloy_ipc");
        if (JS_IsString(ipc_key_val)) {
            JSCStringBuf buf;
            const char *ipc_key = JS_ToCString(ctx, &buf, ipc_key_val);
            JSValue args_val = JS_GetPropertyStr(ctx, value, "args");

            // Build and send IPC request
            AlloyMessage msg;
            msg.type = ALLOY_MSG_BROWSER_API_REQUEST;
            char *args_json = alloy_serialise_args(ctx, 1, &args_val, ARGS_NONE);
            size_t payload_size = strlen(ipc_key) + strlen(args_json) + 32;
            msg.payload = malloc(payload_size);
            snprintf((char *)msg.payload, payload_size, "{\"api\":\"%s\",\"args\":%s}", ipc_key, args_json);
            msg.payload_len = strlen((char *)msg.payload);
            free(args_json);

            char req_id[37];
            snprintf(req_id, sizeof(req_id), "%llu", (unsigned long long)rt->next_outbound_seq);
            alloy_ipc_send(rt, &msg);
            free(msg.payload);

            // Blocking loop
            AlloyMessage resp;
            while (alloy_ipc_try_receive(rt, req_id, &resp) != 0) {
                alloy_poll_io(rt);
                usleep(1000);
            }

            JSValue response_js = alloy_deserialise_result(ctx, &resp, RET_OBJECT);
            res = JS_Call(ctx, next_fn, gen, 1, &response_js);
            JS_FreeValue(ctx, response_js);
            JS_FreeValue(ctx, args_val);
            free(resp.payload); // Fix memory leak
        } else {
            res = JS_Call(ctx, next_fn, gen, 1, &value);
        }
        JS_FreeValue(ctx, ipc_key_val);
        JS_FreeValue(ctx, value);
    }
    return JS_UNDEFINED;
}

static JSValue js_deferred_helper(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    return JS_UNDEFINED; // Stub
}

static JSValue js_off(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    // Implement removal logic
    return JS_UNDEFINED;
}

static JSValue js_emit(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    alloy_runtime_t *rt = JS_GetContextOpaque(ctx);
    JSCStringBuf buf;
    const char *name = JS_ToCString(ctx, &buf, argv[0]);
    if (!name) return JS_EXCEPTION;
    alloy_event_emit(rt, name, argc > 1 ? argv[1] : JS_UNDEFINED);
    return JS_UNDEFINED;
}

static JSValue js_next_tick(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    alloy_runtime_t *rt = JS_GetContextOpaque(ctx);
    if (!JS_IsFunction(ctx, argv[0])) return JS_EXCEPTION;
    if (rt->next_tick_count < 256) {
        rt->next_tick_queue[rt->next_tick_count++] = JS_DupValue(ctx, argv[0]);
    }
    return JS_UNDEFINED;
}

static void alloy_event_emit(alloy_runtime_t *rt, const char *name, JSValue arg) {
    JSCStringBuf buf;
    const char *arg_json = JS_ToCString(rt->mqjs_ctx, &buf, arg);
    alloy_engine_emit(rt->mqjs_engine, name, arg_json ? arg_json : "null");
}

static AlloyEvent *alloy_event_dequeue(alloy_runtime_t *rt) {
    if (rt->event_queue.count == 0) return NULL;
    AlloyEvent *ev = &rt->event_queue.events[rt->event_queue.head];
    rt->event_queue.head = (rt->event_queue.head + 1) % 4096;
    rt->event_queue.count--;
    return ev;
}

static void webview_on_terminated(webview_t w, void *arg) {
    alloy_runtime_t *rt = (alloy_runtime_t *)arg;
    fprintf(stderr, "[alloy] WebViewProcess terminated unexpectedly\n");

    // Attempt recovery
    rt->webview_engine->destroy_fn(rt->webview_engine);
    rt->webview_engine = alloy_webview_engine_create(rt);

    if (!rt->webview_engine) {
        fprintf(stderr, "[alloy] WebViewProcess restart failed - entering degraded mode\n");
    } else {
        // Re-bind all globals from ALLOY_BROWSER_API_TABLE
        for (int i = 0; ALLOY_BROWSER_API_TABLE[i].js_name != NULL; i++) {
            alloy_engine_bind(rt->webview_engine, ALLOY_BROWSER_API_TABLE[i].js_name, (void*)alloy_browser_proxy_dispatch, (void *)&ALLOY_BROWSER_API_TABLE[i]);
        }
    }
}

static void alloy_poll_io(alloy_runtime_t *rt) {
    // Non-blocking webview iteration
    // We assume webview_iterate(rt->webview_engine->w, 0) or similar non-blocking call exists
    // For this mock, we'll just check for pending IPC responses
    // and manually process them if they exist in a real implementation.
}

static void alloy_drain_next_tick(alloy_runtime_t *rt) {
    while (rt->next_tick_count > 0) {
        JSValue fn = rt->next_tick_queue[0];
        // Shift queue (inefficient but safe for small queue)
        for (int i = 0; i < rt->next_tick_count - 1; i++) {
            rt->next_tick_queue[i] = rt->next_tick_queue[i + 1];
        }
        rt->next_tick_count--;

        mprotect(rt->mem_buf, rt->mem_size, PROT_READ | PROT_WRITE);
        JSValue res = JS_Call(rt->mqjs_ctx, fn, JS_UNDEFINED, 0, NULL);
        mprotect(rt->mem_buf, rt->mem_size, PROT_NONE);

        if (JS_IsException(res)) {
            alloy_event_emit(rt, "error", JS_GetException(rt->mqjs_ctx));
        }
        JS_FreeValue(rt->mqjs_ctx, res);
        JS_FreeValue(rt->mqjs_ctx, fn);
    }
}

void alloy_run(alloy_runtime_t *rt) {
    while (!rt->shutdown) {
        alloy_poll_io(rt);

        while (rt->event_queue.count > 0) {
            AlloyEvent *ev = alloy_event_dequeue(rt);
            mprotect(rt->mem_buf, rt->mem_size, PROT_READ | PROT_WRITE);
            JSValue res = JS_Call(rt->mqjs_ctx, ev->listener, JS_UNDEFINED, ev->argc, ev->args);
            mprotect(rt->mem_buf, rt->mem_size, PROT_NONE);

            if (JS_IsException(res)) {
                alloy_event_emit(rt, "error", JS_GetException(rt->mqjs_ctx));
            }
            JS_FreeValue(rt->mqjs_ctx, res);
            for (int i = 0; i < ev->argc; i++) JS_FreeValue(rt->mqjs_ctx, ev->args[i]);
            JS_FreeValue(rt->mqjs_ctx, ev->listener);
            free(ev->name);
        }

        alloy_drain_next_tick(rt);

        // If webview is blocking, this loop won't work as expected
        // webview_run(rt->webview);
    }
}

static void alloy_generic_bind_wrapper(void *opaque, const char *args_json, char *out_result, size_t result_size) {
    alloy_binding_t *b = (alloy_binding_t *)opaque;
    b->fn("generic-id", args_json, b->arg);
    strncpy(out_result, "null", result_size);
}

void alloy_bind_global(alloy_runtime_t *rt, const char *name, alloy_bind_fn_t fn, void *arg) {
    alloy_binding_t *b = malloc(sizeof(alloy_binding_t));
    b->fn = fn; b->arg = arg; b->rt = rt;

    alloy_engine_bind(rt->mqjs_engine, name, alloy_generic_bind_wrapper, b);
    if (rt->webview_engine) {
        alloy_engine_bind(rt->webview_engine, name, alloy_generic_bind_wrapper, b);
    }
}

void alloy_engine_destroy(alloy_runtime_t *rt) {
    if (!rt) return;
    alloy_engine_destroy(rt->mqjs_engine);
    alloy_engine_destroy(rt->webview_engine);
    munmap(rt->mem_buf, rt->mem_size);
    explicit_bzero(rt->session_key, 32);
    free(rt);
}

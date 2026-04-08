#include "engine.h"
#include "ipc.h"
#include "browser_api_table.h"
#include "backend_adapters/mqjs_bridge.h"
#include "../../packages/mquickjs/mquickjs_priv.h"
#include "blake3/blake3.h"

// Forward declarations for missing declarations
AlloyJSEngine *alloy_webview_engine_create(webview_t w, alloy_runtime_t *rt);
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
#include <sys/time.h>
#include <errno.h>

#ifdef __APPLE__
#include <mach-o/dyld.h>
#endif

#define JS_CLASS_COUNT (JS_CLASS_USER + 64)
#define MAX_TIMERS 256


/* timers */
typedef struct {
    int allocated;
    JSGCRef func;
    int64_t timeout; /* in ms */
} JSTimer;

static JSValue js_date_now(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
static JSValue js_print(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
static JSValue js_performance_now(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
static JSValue js_gc(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
static JSValue js_load(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
static JSValue js_setTimeout(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
static JSValue js_clearTimeout(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);

#define js_stdlib js_stdlib_original
#include "../../packages/mquickjs/mqjs_stdlib.h"
#undef js_stdlib

__attribute__((section(".rodata")))
static const uint8_t __alloy_expected_hash[32] = { 0 };

extern JSValue alloy_mqjs_raw_dispatch(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv, JSValue params);

JSValue alloy_runtime_atob(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue alloy_runtime_btoa(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue alloy_runtime_structured_clone(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue alloy_runtime_queue_microtask(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue alloy_runtime_set_interval(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue alloy_runtime_clear_interval(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue alloy_runtime_clear_timeout(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue alloy_runtime_alert(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue alloy_runtime_confirm(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue alloy_runtime_prompt(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue alloy_runtime_report_error(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);

// Missing MQJS functions provided by bridge or implemented here
extern char *alloy_mqjs_to_json(JSContext *ctx, JSValue val);
extern JSValue alloy_mqjs_from_json(JSContext *ctx, const char *json);

static const JSCFunctionDef alloy_c_function_table[] = {
    [JS_CFUNCTION_bound] = { { .generic_params = js_function_bound }, JS_NULL, JS_CFUNC_generic_params, 0, 0 },
    [JS_CFUNCTION_USER + 0] = { { .generic = js_date_now }, JS_NULL, JS_CFUNC_generic, 0, 0 },
    [JS_CFUNCTION_USER + 1] = { { .generic = js_print }, JS_NULL, JS_CFUNC_generic, 1, 0 },
    [JS_CFUNCTION_USER + 2] = { { .generic = js_performance_now }, JS_NULL, JS_CFUNC_generic, 0, 0 },
    [JS_CFUNCTION_USER + 3] = { { .generic = js_gc }, JS_NULL, JS_CFUNC_generic, 0, 0 },
    [JS_CFUNCTION_USER + 4] = { { .generic = js_load }, JS_NULL, JS_CFUNC_generic, 1, 0 },
    [JS_CFUNCTION_USER + 5] = { { .generic = js_setTimeout }, JS_NULL, JS_CFUNC_generic, 2, 0 },
    [JS_CFUNCTION_USER + 6] = { { .generic = js_clearTimeout }, JS_NULL, JS_CFUNC_generic, 1, 0 },

    [JS_CFUNCTION_USER + 11] = { { .generic = alloy_runtime_spawn }, JS_NULL, JS_CFUNC_generic, 3, 0 },
    [JS_CFUNCTION_USER + 12] = { { .generic = alloy_runtime_spawn_sync }, JS_NULL, JS_CFUNC_generic, 3, 0 },
    [JS_CFUNCTION_USER + 28] = { { .generic_params = alloy_mqjs_vtable_dispatch }, JS_NULL, JS_CFUNC_generic_params, 0, 0 },
    [JS_CFUNCTION_USER + 17] = { { .generic = alloy_runtime_atob }, JS_NULL, JS_CFUNC_generic, 1, 0 },
    [JS_CFUNCTION_USER + 18] = { { .generic = alloy_runtime_btoa }, JS_NULL, JS_CFUNC_generic, 1, 0 },
    [JS_CFUNCTION_USER + 19] = { { .generic = alloy_runtime_structured_clone }, JS_NULL, JS_CFUNC_generic, 1, 0 },
    [JS_CFUNCTION_USER + 20] = { { .generic = alloy_runtime_queue_microtask }, JS_NULL, JS_CFUNC_generic, 1, 0 },
    [JS_CFUNCTION_USER + 21] = { { .generic = alloy_runtime_set_interval }, JS_NULL, JS_CFUNC_generic, 2, 0 },
    [JS_CFUNCTION_USER + 22] = { { .generic = alloy_runtime_clear_interval }, JS_NULL, JS_CFUNC_generic, 1, 0 },
    [JS_CFUNCTION_USER + 23] = { { .generic = alloy_runtime_clear_timeout }, JS_NULL, JS_CFUNC_generic, 1, 0 },
    [JS_CFUNCTION_USER + 24] = { { .generic = alloy_runtime_alert }, JS_NULL, JS_CFUNC_generic, 1, 0 },
    [JS_CFUNCTION_USER + 25] = { { .generic = alloy_runtime_confirm }, JS_NULL, JS_CFUNC_generic, 1, 0 },
    [JS_CFUNCTION_USER + 26] = { { .generic = alloy_runtime_prompt }, JS_NULL, JS_CFUNC_generic, 2, 0 },
    [JS_CFUNCTION_USER + 27] = { { .generic = alloy_runtime_report_error }, JS_NULL, JS_CFUNC_generic, 1, 0 },

    [JS_CFUNCTION_USER + 29] = { { .generic = alloy_runtime_secure_eval_internal }, JS_NULL, JS_CFUNC_generic, 1, 0 },
    [JS_CFUNCTION_USER + 30] = { { .generic = alloy_runtime_get_mmap }, JS_NULL, JS_CFUNC_generic, 0, 0 },
    [JS_CFUNCTION_USER + 31] = { { .generic = alloy_runtime_read_mmap }, JS_NULL, JS_CFUNC_generic, 1, 0 },
    [JS_CFUNCTION_USER + 32] = { { .generic = alloy_runtime_on }, JS_NULL, JS_CFUNC_generic, 2, 0 },
    [JS_CFUNCTION_USER + 33] = { { .generic = alloy_runtime_emit }, JS_NULL, JS_CFUNC_generic, 2, 0 },
    [JS_CFUNCTION_USER + 34] = { { .generic = alloy_runtime_next_tick }, JS_NULL, JS_CFUNC_generic, 1, 0 },
    [JS_CFUNCTION_USER + 35] = { { .generic = alloy_runtime_parse_validate }, JS_NULL, JS_CFUNC_generic, 1, 0 },
    [JS_CFUNCTION_USER + 36] = { { .generic = alloy_runtime_await_helper }, JS_NULL, JS_CFUNC_generic, 1, 0 },
    [JS_CFUNCTION_USER + 37] = { { .generic = alloy_runtime_deferred_helper }, JS_NULL, JS_CFUNC_generic, 1, 0 },
    [JS_CFUNCTION_USER + 38] = { { { .generic_params = alloy_mqjs_raw_dispatch } }, JS_NULL, JS_CFUNC_generic_params, 0, 0 },
};

static const JSCFinalizer alloy_c_finalizer_table[JS_CLASS_COUNT - JS_CLASS_USER] = { 0 };

const JSSTDLibraryDef js_stdlib = {
  js_stdlib_table,
  alloy_c_function_table,
  alloy_c_finalizer_table,
  2023,
  64,
  554,
  1934,
  JS_CLASS_COUNT,
};

char *base64_encode(const uint8_t *input, int length);
uint8_t *base64_decode(const char *input, int length, int *out_len);
static void run_timers(JSContext *ctx);
static uint8_t *load_file(const char *filename, int *plen);

void alloy_legacy_handler(AlloyJSEngine *engine, void *opaque, const char *args_json, char *out_result, size_t result_size) {
    JSCFunction *func = (JSCFunction *)opaque;
    alloy_runtime_t *rt = (alloy_runtime_t *)engine->runtime;
    JSContext *ctx = rt->mqjs_ctx;

    mprotect(rt->mem_buf, rt->js_heap_size, PROT_READ | PROT_WRITE);
    JSValue args = alloy_mqjs_from_json(ctx, args_json);

    uint32_t argc = 0;
    if (JS_IsArray(ctx, args)) {
        JSValue len_val = JS_GetPropertyStr(ctx, args, "length");
        JS_ToUint32(ctx, &argc, len_val);
        JS_FreeValue(ctx, len_val);
    }

    JSValue *argv = argc > 0 ? malloc(argc * sizeof(JSValue)) : NULL;
    for (uint32_t i = 0; i < argc; i++) argv[i] = JS_GetPropertyUint32(ctx, args, i);

    JSValue res = func(ctx, JS_UNDEFINED, argc, argv);

    char *json = alloy_mqjs_to_json(ctx, res);
    if (json) {
        strncpy(out_result, json, result_size - 1);
        out_result[result_size - 1] = '\0';
        free(json);
    } else {
        snprintf(out_result, result_size, "null");
    }

    for (uint32_t i = 0; i < argc; i++) JS_FreeValue(ctx, argv[i]);
    if (argv) free(argv);
    JS_FreeValue(ctx, args);
    JS_FreeValue(ctx, res);
    mprotect(rt->mem_buf, rt->js_heap_size, PROT_NONE);
}

static JSValue js_print(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv)
{
    int i;
    JSValue v;
    for(i = 0; i < argc; i++) {
        if (i != 0) putchar(' ');
        v = argv[i];
        if (JS_IsString(ctx, v)) {
            JSCStringBuf buf;
            const char *str;
            size_t len;
            str = JS_ToCStringLen(ctx, &len, v, &buf);
            fwrite(str, 1, len, stdout);
        } else {
            JS_PrintValueF(ctx, argv[i], JS_DUMP_LONG);
        }
    }
    putchar('\n');
    return JS_UNDEFINED;
}

static JSValue js_gc(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv)
{
    JS_GC(ctx);
    return JS_UNDEFINED;
}

static int64_t get_time_ms(void)
{
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (uint64_t)ts.tv_sec * 1000 + (ts.tv_nsec / 1000000);
}

static JSValue js_date_now(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv)
{
    struct timeval tv;
    gettimeofday(&tv, NULL);
    return JS_NewInt64(ctx, (int64_t)tv.tv_sec * 1000 + (tv.tv_usec / 1000));
}

static JSValue js_performance_now(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv)
{
    return JS_NewInt64(ctx, get_time_ms());
}

static JSValue js_load(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv)
{
    const char *filename;
    JSCStringBuf buf_str;
    uint8_t *buf;
    int buf_len;
    JSValue ret;
    filename = JS_ToCString(ctx, argv[0], &buf_str);
    if (!filename) return JS_EXCEPTION;
    buf = load_file(filename, &buf_len);
    ret = JS_Eval(ctx, (const char *)buf, buf_len, filename, 0);
    free(buf);
    return ret;
}

static JSTimer js_timer_list[MAX_TIMERS];

static JSValue js_setTimeout(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv)
{
    JSTimer *th;
    int delay, i;
    JSValue *pfunc;
    if (!JS_IsFunction(ctx, argv[0])) return JS_ThrowTypeError(ctx, "not a function");
    if (JS_ToInt32(ctx, &delay, argv[1])) return JS_EXCEPTION;
    for(i = 0; i < MAX_TIMERS; i++) {
        th = &js_timer_list[i];
        if (!th->allocated) {
            pfunc = JS_AddGCRef(ctx, &th->func);
            *pfunc = argv[0];
            th->timeout = get_time_ms() + delay;
            th->allocated = 1;
            return JS_NewInt32(ctx, i);
        }
    }
    return JS_ThrowInternalError(ctx, "too many timers");
}

static JSValue js_clearTimeout(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv)
{
    int timer_id;
    JSTimer *th;
    if (JS_ToInt32(ctx, &timer_id, argv[0])) return JS_EXCEPTION;
    if (timer_id >= 0 && timer_id < MAX_TIMERS) {
        th = &js_timer_list[timer_id];
        if (th->allocated) {
            JS_DeleteGCRef(ctx, &th->func);
            th->allocated = 0;
        }
    }
    return JS_UNDEFINED;
}

static void run_timers(JSContext *ctx)
{
    int64_t min_delay, delay, cur_time;
    int has_timer, i;
    JSTimer *th;
    struct timespec ts;
    for(;;) {
        min_delay = 1000;
        cur_time = get_time_ms();
        has_timer = 0;
        for(i = 0; i < MAX_TIMERS; i++) {
            th = &js_timer_list[i];
            if (th->allocated) {
                has_timer = 1;
                delay = th->timeout - cur_time;
                if (delay <= 0) {
                    JSValue ret;
                    if (JS_StackCheck(ctx, 2)) return;
                    JS_PushArg(ctx, th->func.val);
                    JS_PushArg(ctx, JS_NULL);
                    JS_DeleteGCRef(ctx, &th->func);
                    th->allocated = 0;
                    ret = JS_Call(ctx, 0);
                    if (JS_IsException(ret)) JS_FreeValue(ctx, JS_GetException(ctx));
                    JS_FreeValue(ctx, ret);
                    min_delay = 0;
                    break;
                } else if (delay < min_delay) {
                    min_delay = delay;
                }
            }
        }
        if (!has_timer) break;
        if (min_delay > 0) {
            ts.tv_sec = min_delay / 1000;
            ts.tv_nsec = (min_delay % 1000) * 1000000;
            nanosleep(&ts, NULL);
        }
    }
}

static uint8_t *load_file(const char *filename, int *plen)
{
    FILE *f = fopen(filename, "rb");
    if (!f) return NULL;
    fseek(f, 0, SEEK_END);
    int buf_len = ftell(f);
    fseek(f, 0, SEEK_SET);
    uint8_t *buf = malloc(buf_len + 1);
    fread(buf, 1, buf_len, f);
    buf[buf_len] = '\0';
    fclose(f);
    if (plen) *plen = buf_len;
    return buf;
}

static int JS_ToBool(JSContext *ctx, JSValue v) {
    if (JS_IsBool(v)) return JS_VALUE_GET_SPECIAL_VALUE(v);
    if (JS_IsNull(v) || JS_IsUndefined(v)) return 0;
    if (JS_IsInt(v)) return JS_VALUE_GET_INT(v) != 0;
    return 1;
}

void alloy_browser_proxy_handler(AlloyJSEngine *engine, void *opaque, const char *args_json, char *out_result, size_t result_size) {
    const AlloyBrowserAPIEntry *entry = (const AlloyBrowserAPIEntry *)opaque;
    alloy_runtime_t *rt = (alloy_runtime_t *)engine->runtime;

    if (rt->webview == NULL) {
        snprintf(out_result, result_size, "{\"error\": \"webview_unavailable\"}");
        return;
    }

    // 2.4 Build AlloyMessage with type ALLOY_MSG_BROWSER_API_REQUEST
    char req_id[32];
    snprintf(req_id, sizeof(req_id), "req-%llu", (unsigned long long)rt->ipc_state.next_outbound_seq);

    char *payload_json = NULL;
    if (asprintf(&payload_json, "{\"api\":\"%s\",\"id\":\"%s\",\"args\":%s}",
             entry->js_name, req_id, args_json) < 0) {
        snprintf(out_result, result_size, "null");
        return;
    }

    // 2.5 Call through the WebView engine abstraction
    char *result_json = NULL;
    int status = alloy_engine_call(rt->webview_engine, entry->webview_method, payload_json, &result_json);
    free(payload_json);

    if (status != 0) {
        snprintf(out_result, result_size, "{\"error\": \"Browser API call failed\"}");
    } else if (result_json) {
        strncpy(out_result, result_json, result_size - 1);
        out_result[result_size - 1] = '\0';
        free(result_json);
    } else {
        snprintf(out_result, result_size, "null");
    }
}

JSValue alloy_runtime_atob(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_EXCEPTION;
    JSCStringBuf buf;
    const char *str = JS_ToCString(ctx, argv[0], &buf);
    if (!str) return JS_EXCEPTION;
    int out_len;
    uint8_t *decoded = base64_decode(str, strlen(str), &out_len);
    JSValue res = JS_NewStringLen(ctx, (char *)decoded, out_len);
    free(decoded);
    return res;
}

JSValue alloy_runtime_btoa(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_EXCEPTION;
    JSCStringBuf buf;
    const char *str = JS_ToCString(ctx, argv[0], &buf);
    if (!str) return JS_EXCEPTION;
    char *encoded = base64_encode((uint8_t *)str, strlen(str));
    JSValue res = JS_NewString(ctx, encoded);
    free(encoded);
    return res;
}

JSValue alloy_runtime_structured_clone(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_EXCEPTION;
    JSValue global = JS_GetGlobalObject(ctx);
    JSValue json = JS_GetPropertyStr(ctx, global, "JSON");
    JSValue stringify = JS_GetPropertyStr(ctx, json, "stringify");
    JSValue parse = JS_GetPropertyStr(ctx, json, "parse");
    JSValue str = JS_Call(ctx, stringify, json, 1, &argv[0]);
    JSValue clone = JS_Call(ctx, parse, json, 1, &str);
    JS_FreeValue(ctx, str); JS_FreeValue(ctx, parse); JS_FreeValue(ctx, stringify); JS_FreeValue(ctx, json); JS_FreeValue(ctx, global);
    return clone;
}

JSValue alloy_runtime_queue_microtask(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) {
    alloy_runtime_t *rt = JS_GetContextOpaque(ctx);
    if (rt->next_tick_count < 256) rt->next_tick_queue[rt->next_tick_count++] = argv[0];
    return JS_UNDEFINED;
}

JSValue alloy_runtime_set_interval(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) { return JS_NewInt32(ctx, 0); }
JSValue alloy_runtime_clear_interval(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) { return JS_UNDEFINED; }
JSValue alloy_runtime_clear_timeout(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) { return JS_UNDEFINED; }
JSValue alloy_runtime_alert(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) { return JS_UNDEFINED; }
JSValue alloy_runtime_confirm(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) { return JS_FALSE; }
JSValue alloy_runtime_prompt(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) { return JS_NULL; }
JSValue alloy_runtime_report_error(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) { return JS_UNDEFINED; }

JSValue alloy_runtime_spawn(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_ThrowTypeError(ctx, "Missing command");
    const char *cmd = JS_ToCString(ctx, argv[0], NULL);
    if (!cmd) return JS_EXCEPTION;

    JSValue res = JS_NewObject(ctx);
    pid_t pid = fork();
    if (pid == 0) {
        // Child: handle args etc. Simplified for foundation.
        execlp(cmd, cmd, (char *)NULL);
        _exit(127);
    } else if (pid > 0) {
        JS_SetPropertyStr(ctx, res, "pid", JS_NewInt32(ctx, pid));
        JS_SetPropertyStr(ctx, res, "stdout", JS_NULL);
        JS_SetPropertyStr(ctx, res, "stderr", JS_NULL);
        JS_SetPropertyStr(ctx, res, "exitCode", JS_NULL);
        JS_FreeCString(ctx, cmd);
        return res;
    } else {
        JS_FreeCString(ctx, cmd);
        return JS_ThrowInternalError(ctx, "fork failed: %s", strerror(errno));
    }
}

JSValue alloy_runtime_spawn_sync(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_ThrowTypeError(ctx, "Missing command");
    const char *cmd = JS_ToCString(ctx, argv[0], NULL);
    if (!cmd) return JS_EXCEPTION;

    int status = system(cmd);
    JSValue res = JS_NewObject(ctx);
    JS_SetPropertyStr(ctx, res, "exitCode", JS_NewInt32(ctx, WEXITSTATUS(status)));
    JS_SetPropertyStr(ctx, res, "stdout", JS_NULL);
    JS_SetPropertyStr(ctx, res, "stderr", JS_NULL);
    JS_FreeCString(ctx, cmd);
    return res;
}

JSValue alloy_runtime_secure_eval_internal(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_EXCEPTION;
    const char *script = JS_ToCString(ctx, argv[0], NULL);
    if (!script) return JS_EXCEPTION;
    alloy_runtime_t *rt = JS_GetContextOpaque(ctx);
    mprotect(rt->mem_buf, rt->js_heap_size, PROT_READ | PROT_WRITE);
    JSValue res = JS_Eval(ctx, script, strlen(script), "<secureEval>", JS_EVAL_RETVAL);
    mprotect(rt->mem_buf, rt->js_heap_size, PROT_NONE);
    JS_FreeCString(ctx, script);
    return res;
}

JSValue alloy_runtime_parse_validate(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_EXCEPTION;
    const char *code = JS_ToCString(ctx, argv[0], NULL);
    if (!code) return JS_EXCEPTION;
    JSValue res = JS_Parse(ctx, code, strlen(code), "<transpile>", 0);
    if (JS_IsException(res)) {
        JSValue err = JS_GetException(ctx);
        JS_FreeCString(ctx, code);
        return err;
    }
    JS_FreeValue(ctx, res);
    JS_FreeCString(ctx, code);
    return JS_NULL;
}

JSValue alloy_runtime_await_helper(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_EXCEPTION;
    JSValue gen = argv[0];
    JSValue next_name = JS_NewAtom(ctx, "next");
    JSValue next_func = JS_GetProperty(ctx, gen, next_name);
    JS_FreeAtom(ctx, next_name);

    if (!JS_IsFunction(ctx, next_func)) {
        JS_FreeValue(ctx, next_func);
        return JS_DupValue(ctx, gen);
    }

    JSValue val = JS_UNDEFINED;
    alloy_runtime_t *rt = JS_GetContextOpaque(ctx);

    while (!rt->shutdown) {
        JS_PushArg(ctx, val);
        JS_PushArg(ctx, next_func);
        JS_PushArg(ctx, gen);
        JSValue res = JS_Call(ctx, 1);
        val = JS_UNDEFINED;
        if (JS_IsException(res)) {
            JS_FreeValue(ctx, next_func);
            return res;
        }

        JSValue done_val = JS_GetPropertyStr(ctx, res, "done");
        bool done = JS_ToBool(ctx, done_val);
        JS_FreeValue(ctx, done_val);

        JSValue value = JS_GetPropertyStr(ctx, res, "value");
        JS_FreeValue(ctx, res);

        if (done) {
            JS_FreeValue(ctx, next_func);
            return value;
        }

        // yielded { __alloy_ipc: "...", args: [...] }
        JSValue ipc_name_val = JS_GetPropertyStr(ctx, value, "__alloy_ipc");
        if (JS_IsUndefined(ipc_name_val)) {
            val = value; // Regular yield
            continue;
        }

        const char *api = JS_ToCString(ctx, ipc_name_val, NULL);
        JSValue args_val = JS_GetPropertyStr(ctx, value, "args");
        char *args_json = alloy_mqjs_to_json(ctx, args_val);
        JS_FreeValue(ctx, args_val);
        JS_FreeCString(ctx, api);
        JS_FreeValue(ctx, ipc_name_val);
        JS_FreeValue(ctx, value);

        // Drive IPC synchronously (Requirement 4.5)
        char *result_json = NULL;
        alloy_engine_call(rt->webview_engine, api, args_json, &result_json);
        free(args_json);

        val = JS_ParseJSON(ctx, result_json, strlen(result_json), "<ipc_res>");
        free(result_json);
    }

    JS_FreeValue(ctx, next_func);
    return JS_UNDEFINED;
}

JSValue alloy_runtime_deferred_helper(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    // Synchronous deferred wrapper
    // Drives executor immediately
    if (argc < 1) return JS_EXCEPTION;
    JSValue executor = argv[0];

    // In MQJS sync model, this just calls the executor
    // and returns a mock promise-like object
    JSValue res = JS_NewObject(ctx);
    JS_SetPropertyStr(ctx, res, "then", JS_NewCFunction(ctx, alloy_runtime_await_helper, "then", 1));

    JSValue resolve = JS_NewObject(ctx); // dummy
    JSValue reject = JS_NewObject(ctx); // dummy
    JSValue args[2] = { resolve, reject };
    JS_PushArg(ctx, resolve);
    JS_PushArg(ctx, reject);
    JS_PushArg(ctx, executor);
    JS_PushArg(ctx, JS_UNDEFINED);
    JSValue call_res = JS_Call(ctx, 2);
    JS_FreeValue(ctx, call_res);
    JS_FreeValue(ctx, resolve); JS_FreeValue(ctx, reject);

    return res;
}
static void alloy_mmap_hash_stable_regions(AlloyMmapSnapshot *snap) {
    blake3_hasher h;

    // Hash .text contents
    for (int i = 0; i < snap->count; i++) {
        if (snap->regions[i].kind == ALLOY_REGION_CODE && snap->regions[i].readable) {
            blake3_hasher_init(&h);
            blake3_hasher_update(&h, (void*)snap->regions[i].start, snap->regions[i].size);
            blake3_hasher_finalize(&h, snap->blake3_text, 32);
            break;
        }
    }

    // Hash .rodata contents
    for (int i = 0; i < snap->count; i++) {
        if (snap->regions[i].kind == ALLOY_REGION_RODATA && snap->regions[i].readable) {
            blake3_hasher_init(&h);
            blake3_hasher_update(&h, (void*)snap->regions[i].start, snap->regions[i].size);
            blake3_hasher_finalize(&h, snap->blake3_rodata, 32);
            break;
        }
    }

    // Hash .text + .rodata concatenated
    blake3_hasher_init(&h);
    bool has_text = false, has_rodata = false;
    for (int i = 0; i < snap->count; i++) {
        if (snap->regions[i].kind == ALLOY_REGION_CODE && snap->regions[i].readable) {
            blake3_hasher_update(&h, (void*)snap->regions[i].start, snap->regions[i].size);
            has_text = true;
        }
        if (snap->regions[i].kind == ALLOY_REGION_RODATA && snap->regions[i].readable) {
            blake3_hasher_update(&h, (void*)snap->regions[i].start, snap->regions[i].size);
            has_rodata = true;
        }
    }
    if (has_text || has_rodata) {
        blake3_hasher_finalize(&h, snap->blake3_text_rodata, 32);
    } else {
        memset(snap->blake3_text_rodata, 0, 32);
    }
}

int alloy_mmap_enumerate(alloy_runtime_t *rt, AlloyMmapSnapshot *snap) {
#if defined(__linux__)
    FILE *f = fopen("/proc/self/maps", "r");
    if (!f) return -1;
    char line[512];
    snap->count = 0;

    while (fgets(line, sizeof(line), f) && snap->count < ALLOY_MMAP_MAX_REGIONS) {
        AlloyMmapRegion *r = &snap->regions[snap->count];
        uintptr_t start, end;
        char perms[8], file[256] = "";
        int fields = sscanf(line, "%lx-%lx %4s %*x %*s %*d %255[^\n]", &start, &end, perms, file);

        r->start = start;
        r->size  = end - start;
        strncpy(r->perms, perms, sizeof(r->perms));
        r->readable   = perms[0] == 'r';
        r->writable   = perms[1] == 'w';
        r->executable = perms[2] == 'x';

        // Classify region
        if (strcmp(file, "[stack]") == 0) {
            r->kind = ALLOY_REGION_STACK;
            strncpy(r->name, "[stack]", sizeof(r->name));
            r->stable = false;
        } else if (strcmp(file, "[heap]") == 0) {
            r->kind = ALLOY_REGION_HEAP;
            strncpy(r->name, "[heap]", sizeof(r->name));
            r->stable = false;
        } else if (strcmp(file, "[vdso]") == 0) {
            r->kind = ALLOY_REGION_VDSO;
            strncpy(r->name, "[vdso]", sizeof(r->name));
            r->stable = false;
        } else if (start == (uintptr_t)rt->mem_buf) {
            r->kind = ALLOY_REGION_JS_ARENA;
            strncpy(r->name, "js_arena", sizeof(r->name));
            r->stable = false;
        } else if (r->executable && strlen(file) > 0) {
            r->kind = ALLOY_REGION_CODE;
            strncpy(r->name, ".text", sizeof(r->name));
            strncpy(r->file, file, sizeof(r->file));
            r->stable = true;
        } else if (!r->writable && !r->executable && strlen(file) > 0) {
            r->kind = ALLOY_REGION_RODATA;
            strncpy(r->name, ".rodata", sizeof(r->name));
            strncpy(r->file, file, sizeof(r->file));
            r->stable = true;
        } else if (r->writable && strlen(file) > 0) {
            r->kind = ALLOY_REGION_DATA;
            strncpy(r->name, ".data", sizeof(r->name));
            r->stable = false;
        } else {
            r->kind = ALLOY_REGION_ANON;
            strncpy(r->name, "anon", sizeof(r->name));
            r->stable = false;
        }
        snap->count++;
    }
    fclose(f);
#else
    // Mock for other platforms
    snap->count = 0;
#endif

    snap->aslr_offset = 0; // Simplified
    alloy_mmap_hash_stable_regions(snap);
    snap->js_arena_protected = (rt->js_heap_prot == PROT_NONE);

    return 0;
}

static char *hex_encode(const uint8_t *data, size_t len) {
    char *hex = malloc(len * 2 + 1);
    for (size_t i = 0; i < len; i++) sprintf(hex + i * 2, "%02x", data[i]);
    hex[len * 2] = '\0';
    return hex;
}

JSValue alloy_runtime_get_mmap(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    alloy_runtime_t *rt = JS_GetContextOpaque(ctx);
    AlloyMmapSnapshot snap;
    alloy_mmap_enumerate(rt, &snap);

    JSValue res = JS_NewObject(ctx);
    JSValue regions = JS_NewArray(ctx, snap.count);
    for (int i = 0; i < snap.count; i++) {
        JSValue r = JS_NewObject(ctx);
        JS_SetPropertyStr(ctx, r, "name", JS_NewString(ctx, snap.regions[i].name));
        JS_SetPropertyStr(ctx, r, "start", JS_NewFloat64(ctx, (double)snap.regions[i].start));
        JS_SetPropertyStr(ctx, r, "size", JS_NewFloat64(ctx, (double)snap.regions[i].size));
        JS_SetPropertyStr(ctx, r, "perms", JS_NewString(ctx, snap.regions[i].perms));
        JS_SetPropertyStr(ctx, r, "readable", JS_NewBool(snap.regions[i].readable));
        JS_SetPropertyStr(ctx, r, "writable", JS_NewBool(snap.regions[i].writable));
        JS_SetPropertyStr(ctx, r, "executable", JS_NewBool(snap.regions[i].executable));
        JS_SetPropertyStr(ctx, r, "stable", JS_NewBool(snap.regions[i].stable));
        JS_SetPropertyUint32(ctx, regions, i, r);
    }
    JS_SetPropertyStr(ctx, res, "regions", regions);

    JSValue blake3 = JS_NewObject(ctx);
    char *text_h = hex_encode(snap.blake3_text, 32);
    char *rodata_h = hex_encode(snap.blake3_rodata, 32);
    char *both_h = hex_encode(snap.blake3_text_rodata, 32);
    JS_SetPropertyStr(ctx, blake3, "text", JS_NewString(ctx, text_h));
    JS_SetPropertyStr(ctx, blake3, "rodata", JS_NewString(ctx, rodata_h));
    JS_SetPropertyStr(ctx, blake3, "textAndRodata", JS_NewString(ctx, both_h));
    free(text_h); free(rodata_h); free(both_h);
    JS_SetPropertyStr(ctx, res, "blake3", blake3);

    JS_SetPropertyStr(ctx, res, "jsArenaProtected", JS_NewBool(snap.js_arena_protected));
    JS_SetPropertyStr(ctx, res, "aslrOffset", JS_NewFloat64(ctx, (double)snap.aslr_offset));

    return res;
}

JSValue alloy_runtime_read_mmap(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    alloy_runtime_t *rt = JS_GetContextOpaque(ctx);
    const char *name = JS_ToCString(ctx, argv[0], NULL);

    AlloyMmapSnapshot snap;
    alloy_mmap_enumerate(rt, &snap);

    for (int i = 0; i < snap.count; i++) {
        if (strcmp(snap.regions[i].name, name) == 0) {
            if (!snap.regions[i].readable) {
                JS_FreeCString(ctx, name);
                return JS_ThrowRangeError(ctx, "region is not readable");
            }
            if (snap.regions[i].kind == ALLOY_REGION_JS_ARENA && snap.js_arena_protected) {
                JS_FreeCString(ctx, name);
                return JS_ThrowRangeError(ctx, "js_arena is currently protected");
            }

            // MQJS does not expose JS_NewArrayBuffer directly to user code easily
            // We'll return an array of bytes for now as an MVP.
            JSValue arr = JS_NewArray(ctx, snap.regions[i].size);
            uint8_t *ptr = (uint8_t *)snap.regions[i].start;
            for (size_t j = 0; j < snap.regions[i].size; j++) {
                JS_SetPropertyUint32(ctx, arr, j, JS_NewInt32(ctx, ptr[j]));
            }
            JS_FreeCString(ctx, name);
            return arr;
        }
    }

    JS_FreeCString(ctx, name);
    return JS_ThrowRangeError(ctx, "region not found");
}

void alloy_event_emit(alloy_runtime_t *rt, const char *name, JSValue arg) {
    for (int i = 0; i < 32; i++) {
        if (rt->listener_registry[i].event_name && strcmp(rt->listener_registry[i].event_name, name) == 0) {
            for (int j = 0; j < rt->listener_registry[i].count; j++) {
                if (rt->event_queue.count < ALLOY_EVENT_QUEUE_CAPACITY) {
                    AlloyEvent *ev = &rt->event_queue.events[rt->event_queue.tail];
                    ev->name = strdup(name);
                    ev->listener = JS_DupValue(rt->mqjs_ctx, rt->listener_registry[i].listeners[j]);
                    ev->args[0] = JS_DupValue(rt->mqjs_ctx, arg);
                    ev->argc = 1;
                    rt->event_queue.tail = (rt->event_queue.tail + 1) % ALLOY_EVENT_QUEUE_CAPACITY;
                    rt->event_queue.count++;
                }
            }
        }
    }
}

AlloyEvent *alloy_event_dequeue(alloy_runtime_t *rt) {
    if (rt->event_queue.count == 0) return NULL;
    AlloyEvent *ev = &rt->event_queue.events[rt->event_queue.head];
    rt->event_queue.head = (rt->event_queue.head + 1) % ALLOY_EVENT_QUEUE_CAPACITY;
    rt->event_queue.count--;
    return ev;
}

void alloy_poll_io(alloy_runtime_t *rt) {
    // Check for pending WebView callbacks/events
    if (rt->webview) {
        // webview_poll is hypothetical for some backends,
        // usually webview_run blocks. In a cooperative loop:
        // webview_dispatch(rt->webview);
    }

    // Check timers
    // Simplified timer check for foundation
}

void alloy_drain_next_tick(alloy_runtime_t *rt) {
    while (rt->next_tick_count > 0) {
        JSValue func = rt->next_tick_queue[0];
        for (int i = 0; i < rt->next_tick_count - 1; i++) rt->next_tick_queue[i] = rt->next_tick_queue[i+1];
        rt->next_tick_count--;
        mprotect(rt->mem_buf, rt->js_heap_size, PROT_READ | PROT_WRITE);
        JS_PushArg(rt->mqjs_ctx, func);
        JS_PushArg(rt->mqjs_ctx, JS_UNDEFINED);
        JSValue res = JS_Call(rt->mqjs_ctx, 0);
        mprotect(rt->mem_buf, rt->js_heap_size, PROT_NONE);
    }
}

JSValue alloy_runtime_on(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    alloy_runtime_t *rt = JS_GetContextOpaque(ctx);
    const char *name = JS_ToCString(ctx, argv[0], NULL);
    JSValue listener = argv[1];
    for (int i = 0; i < 32; i++) {
        if (rt->listener_registry[i].event_name == NULL || strcmp(rt->listener_registry[i].event_name, name) == 0) {
            if (rt->listener_registry[i].event_name == NULL) rt->listener_registry[i].event_name = strdup(name);
            if (rt->listener_registry[i].count < 8) {
                rt->listener_registry[i].listeners[rt->listener_registry[i].count++] = JS_DupValue(ctx, listener);
            }
            break;
        }
    }
    JS_FreeCString(ctx, name);
    return JS_UNDEFINED;
}

JSValue alloy_runtime_off(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) { return JS_UNDEFINED; }
JSValue alloy_runtime_emit(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    alloy_runtime_t *rt = JS_GetContextOpaque(ctx);
    const char *name = JS_ToCString(ctx, argv[0], NULL);
    alloy_event_emit(rt, name, argv[1]);
    JS_FreeCString(ctx, name);
    return JS_UNDEFINED;
}

JSValue alloy_runtime_next_tick(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    alloy_runtime_t *rt = JS_GetContextOpaque(ctx);
    if (rt->next_tick_count < 256) rt->next_tick_queue[rt->next_tick_count++] = JS_DupValue(ctx, argv[0]);
    return JS_UNDEFINED;
}

void alloy_get_exe_path(char *buf, size_t len) {
#if defined(__linux__)
    ssize_t n = readlink("/proc/self/exe", buf, len - 1);
    if (n >= 0) buf[n] = '\0';
#elif defined(__APPLE__)
    uint32_t sz = (uint32_t)len;
    _NSGetExecutablePath(buf, &sz);
#elif defined(_WIN32)
    GetModuleFileNameA(NULL, buf, (DWORD)len);
#endif
}

int alloy_self_attest(void) {
    char exe_path[4096];
    alloy_get_exe_path(exe_path, sizeof(exe_path));
    FILE *f = fopen(exe_path, "rb");
    if (!f) return -1;
    fseek(f, 0, SEEK_END);
    long binary_len = ftell(f);
    fseek(f, 0, SEEK_SET);
    uint8_t *binary = malloc(binary_len);
    fread(binary, 1, binary_len, f);
    fclose(f);
    size_t hash_offset = 0;
    for (size_t i = 0; i < binary_len - 32; i++) {
        if (memcmp(binary + i, __alloy_expected_hash, 32) == 0 && __alloy_expected_hash[0] != 0) {
            hash_offset = i;
            break;
        }
    }
    uint8_t saved[32];
    memcpy(saved, binary + hash_offset, 32);
    memset(binary + hash_offset, 0, 32);
    uint8_t computed[32];
    blake3_hasher hasher;
    blake3_hasher_init(&hasher);
    blake3_hasher_update(&hasher, binary, binary_len);
    blake3_hasher_finalize(&hasher, computed, 32);
    free(binary);
    int diff = 0;
    for (int i = 0; i < 32; i++) diff |= (computed[i] ^ saved[i]);
    return diff == 0 ? 0 : -1;
}

extern void alloy_plugin_register(RuntimePlugin *plugin);
extern RuntimePlugin **alloy_plugin_registry_sorted(void);
extern RuntimePlugin alloy_config_plugin;
extern RuntimePlugin alloy_web_plugin;
extern RuntimePlugin alloy_shell_plugin;
extern RuntimePlugin alloy_async_plugin;

void alloy_engine_bootstrap(alloy_runtime_t *rt) {
    JSContext *ctx = rt->mqjs_ctx;
    mprotect(rt->mem_buf, rt->js_heap_size, PROT_READ | PROT_WRITE);

    // Register core plugins
    alloy_plugin_register(&alloy_config_plugin);
    alloy_plugin_register(&alloy_web_plugin);
    alloy_plugin_register(&alloy_shell_plugin);
    alloy_plugin_register(&alloy_async_plugin);

    RuntimePlugin **sorted = alloy_plugin_registry_sorted();
    for (int i = 0; sorted[i] != NULL; i++) {
        if (sorted[i]->on_init) sorted[i]->on_init(rt->mqjs_engine);
    }
    free(sorted);

    JSValue global_obj = JS_GetGlobalObject(ctx);
    JSValue alloy_obj = JS_GetPropertyStr(ctx, global_obj, "Alloy");
    if (JS_IsUndefined(alloy_obj)) {
        alloy_obj = JS_NewObject(ctx);
        JS_SetPropertyStr(ctx, global_obj, "Alloy", JS_DupValue(ctx, alloy_obj));
    }
    JS_SetPropertyStr(ctx, alloy_obj, "target", JS_NewString(ctx, rt->target == ALLOY_TARGET_BROWSER ? "browser" : "native"));

    JSValue internal_obj = JS_GetPropertyStr(ctx, global_obj, "AlloyInternal");
    if (JS_IsUndefined(internal_obj)) {
        internal_obj = JS_NewObject(ctx);
        JS_SetPropertyStr(ctx, global_obj, "AlloyInternal", JS_DupValue(ctx, internal_obj));
    }
    JS_SetPropertyStr(ctx, internal_obj, "parse", JS_NewCFunctionParams(ctx, JS_CFUNCTION_USER + 35, JS_NULL));
    JS_SetPropertyStr(ctx, internal_obj, "__alloy_await", JS_NewCFunctionParams(ctx, JS_CFUNCTION_USER + 36, JS_NULL));
    JS_SetPropertyStr(ctx, internal_obj, "__alloy_deferred", JS_NewCFunctionParams(ctx, JS_CFUNCTION_USER + 37, JS_NULL));

    JS_FreeValue(ctx, internal_obj);
    JS_FreeValue(ctx, alloy_obj);
    JS_FreeValue(ctx, global_obj);
    mprotect(rt->mem_buf, rt->js_heap_size, PROT_NONE);
}

void alloy_webview_watchdog(webview_t w, void *arg) {
    alloy_runtime_t *rt = (alloy_runtime_t *)arg;
    fprintf(stderr, "[alloy] WebViewProcess terminated unexpectedly\n");

    // Attempt restart
    rt->webview = webview_create(0, NULL);
    if (rt->webview) {
        webview_set_size(rt->webview, 1, 1, WEBVIEW_HINT_MIN);
        // Re-inject bootstrap JS with session key (simplified here)
        rt->webview_engine = alloy_webview_engine_create(rt->webview, rt);
        alloy_engine_bootstrap(rt);
    } else {
        fprintf(stderr, "[alloy] WebViewProcess restart failed — entering degraded mode\n");
        rt->webview_engine = NULL;
    }
}

alloy_runtime_t *alloy_engine_create(void) {
    alloy_runtime_t *rt = calloc(1, sizeof(alloy_runtime_t));
    if (!rt) return NULL;

    rt->target = ALLOY_TARGET_NATIVE;
    rt->js_heap_size = 16 << 20;
    rt->mem_buf = mmap(NULL, rt->js_heap_size, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);

    rt->mqjs_ctx = JS_NewContext(rt->mem_buf, rt->js_heap_size, &js_stdlib);
    JS_SetContextOpaque(rt->mqjs_ctx, rt);
    rt->mqjs_engine = alloy_mqjs_engine_create(rt->mqjs_ctx, rt);

    mprotect(rt->mem_buf, rt->js_heap_size, PROT_NONE);

    rt->webview = webview_create(0, NULL);
    if (rt->webview) {
        webview_set_size(rt->webview, 1, 1, WEBVIEW_HINT_MIN);
        rt->webview_engine = alloy_webview_engine_create(rt->webview, rt);
        // webview_set_exit_callback(rt->webview, alloy_webview_watchdog, rt); // Platform specific or hypothetical
    }

    RAND_bytes(rt->ipc_state.session_key, 32);
    mlock(rt->ipc_state.session_key, 32);
    rt->ipc_state.next_outbound_seq = 1;
    rt->ipc_state.last_inbound_seq = 0;

    alloy_engine_bootstrap(rt);

    return rt;
}

void alloy_engine_free(alloy_runtime_t *rt) {
    if (!rt) return;
    if (rt->webview_engine) alloy_engine_destroy(rt->webview_engine);
    if (rt->mqjs_engine) alloy_engine_destroy(rt->mqjs_engine);
    explicit_bzero(rt->ipc_state.session_key, 32);
    munmap(rt->mem_buf, rt->js_heap_size);
    free(rt);
}

void alloy_run(alloy_runtime_t *rt) {
    while (!rt->shutdown) {
        // 1. Poll OS I/O (non-blocking)
        alloy_poll_io(rt);

        // 2. Drain event queue
        while (rt->event_queue.count > 0) {
            AlloyEvent *ev = alloy_event_dequeue(rt);

            // Open JS heap for this dispatch
            mprotect(rt->mem_buf, rt->js_heap_size, PROT_READ | PROT_WRITE);

            for (int i = 0; i < ev->argc; i++) JS_PushArg(rt->mqjs_ctx, ev->args[i]);
            JS_PushArg(rt->mqjs_ctx, ev->listener);
            JS_PushArg(rt->mqjs_ctx, JS_UNDEFINED);
            JSValue res = JS_Call(rt->mqjs_ctx, ev->argc);

            // Close JS heap immediately after dispatch
            mprotect(rt->mem_buf, rt->js_heap_size, PROT_NONE);

            if (JS_IsException(res)) {
                alloy_event_emit(rt, "error", JS_GetException(rt->mqjs_ctx));
            }

            JS_FreeValue(rt->mqjs_ctx, res);
            JS_FreeValue(rt->mqjs_ctx, ev->listener);
            for (int i = 0; i < ev->argc; i++) JS_FreeValue(rt->mqjs_ctx, ev->args[i]);
            free(ev->name);
        }

        // 3. nextTick queue (runs before next I/O poll)
        alloy_drain_next_tick(rt);

        // If no more events, run webview loop which might block
        if (rt->event_queue.count == 0 && rt->webview) {
            webview_run(rt->webview);
        }
    }
}

JSValue alloy_secure_eval(alloy_runtime_t *rt, const char *script) {
    mprotect(rt->mem_buf, rt->js_heap_size, PROT_READ | PROT_WRITE);
    JSValue res = JS_Eval(rt->mqjs_ctx, script, strlen(script), "<secureEval>", JS_EVAL_RETVAL);
    mprotect(rt->mem_buf, rt->js_heap_size, PROT_NONE);
    return res;
}

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
            } else triple <<= 6;
        }
        out[j++] = (triple >> 16) & 0xFF;
        out[j++] = (triple >> 8) & 0xFF;
        out[j++] = triple & 0xFF;
    }
    *out_len = j;
    return out;
}

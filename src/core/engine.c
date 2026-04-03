#include "engine.h"
#include "mqjs_stdlib.h"
#include "bridge.h"
#include "ipc.h"
#include "browser_api_table.h"
#include "backend_adapters/mqjs_bridge.h"
#include "../../packages/mquickjs/mquickjs_priv.h"
#include "blake3/blake3.h"
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

#define JS_CLASS_COUNT (JS_CLASS_USER + 16)
#define js_stdlib js_stdlib_original
#include "../../packages/mquickjs/mqjs_stdlib.h"
#undef js_stdlib

__attribute__((section(".rodata")))
static const uint8_t __alloy_expected_hash[32] = { 0 };

static JSValue alloy_browser_proxy_dispatch(JSContext *ctx, JSValue this_val, int argc, JSValue *argv, JSValue opaque);

static const JSCFunctionDef alloy_c_function_table[] = {
    [JS_CFUNCTION_bound] = { { .generic_params = js_function_bound }, JS_NULL, JS_CFUNC_generic_params, 0, 0 },
    [JS_CFUNCTION_USER + 0] = { { .generic = js_create_sink }, JS_NULL, JS_CFUNC_generic, 0, 0 },
    [JS_CFUNCTION_USER + 1] = { { .generic = js_sink_start }, JS_NULL, JS_CFUNC_generic, 1, 0 },
    [JS_CFUNCTION_USER + 2] = { { .generic = js_sink_write }, JS_NULL, JS_CFUNC_generic, 1, 0 },
    [JS_CFUNCTION_USER + 3] = { { .generic = js_sink_flush }, JS_NULL, JS_CFUNC_generic, 0, 0 },
    [JS_CFUNCTION_USER + 4] = { { .generic = js_sink_end }, JS_NULL, JS_CFUNC_generic, 0, 0 },
    [JS_CFUNCTION_USER + 5] = { { .generic = js_ipc_send }, JS_NULL, JS_CFUNC_generic, 1, 0 },
    [JS_CFUNCTION_USER + 6] = { { .generic = js_did_generate_keypair }, JS_NULL, JS_CFUNC_generic, 0, 0 },
    [JS_CFUNCTION_USER + 7] = { { .generic = js_print }, JS_NULL, JS_CFUNC_generic, 1, 0 },
    [JS_CFUNCTION_USER + 8] = { { .generic = js_sqlite_open }, JS_NULL, JS_CFUNC_generic, 1, 0 },
    [JS_CFUNCTION_USER + 9] = { { .generic = js_sqlite_run }, JS_NULL, JS_CFUNC_generic, 2, 0 },
    [JS_CFUNCTION_USER + 10] = { { .generic = js_sqlite_close }, JS_NULL, JS_CFUNC_generic, 1, 0 },
    [JS_CFUNCTION_USER + 11] = { { .generic = js_process_spawn }, JS_NULL, JS_CFUNC_generic, 1, 0 },
    [JS_CFUNCTION_USER + 12] = { { .generic = js_process_spawn_sync }, JS_NULL, JS_CFUNC_generic, 1, 0 },
    [JS_CFUNCTION_USER + 13] = { { .generic = js_date_now }, JS_NULL, JS_CFUNC_generic, 0, 0 },
    [JS_CFUNCTION_USER + 14] = { { .generic = js_performance_now }, JS_NULL, JS_CFUNC_generic, 0, 0 },
    [JS_CFUNCTION_USER + 15] = { { .generic = js_setTimeout }, JS_NULL, JS_CFUNC_generic, 2, 0 },
    [JS_CFUNCTION_USER + 16] = { { .generic_params = alloy_browser_proxy_dispatch }, JS_NULL, JS_CFUNC_generic_params, 0, 0 },
};

static const JSCFinaliser alloy_c_finaliser_table[JS_CLASS_COUNT - JS_CLASS_USER] = { 0 };

const JSSTDLibraryDef js_stdlib = {
  js_stdlib_table,
  alloy_c_function_table,
  alloy_c_finaliser_table,
  2023,
  64,
  554,
  1934,
  JS_CLASS_COUNT,
};

static JSValue alloy_browser_proxy_dispatch(JSContext *ctx, JSValue this_val, int argc, JSValue *argv, JSValue opaque) {
    const AlloyBrowserAPIEntry *entry = (const AlloyBrowserAPIEntry *)(uintptr_t)(opaque & ~1);
    alloy_runtime_t *rt = JS_GetContextOpaque(ctx);
    if (!rt) return JS_EXCEPTION;
    if (rt->webview == NULL) {
        return JS_NewString(ctx, "{\"error\": \"webview_unavailable\"}");
    }
    const char *args_json = "[]";
    AlloyMessage msg;
    msg.type = ALLOY_MSG_BROWSER_API_REQUEST;
    char *payload = NULL;
    asprintf(&payload, "{\"api\":\"%s\",\"args\":%s}", entry->js_name, args_json);
    msg.payload_len = strlen(payload);
    msg.payload = (uint8_t *)payload;
    alloy_ipc_send(rt, &msg);
    free(payload);

    // Blocking wait for response
    while (!rt->shutdown) {
        alloy_poll_io(rt);
        if (rt->event_queue.count > 0) {
            AlloyEvent *ev = alloy_event_dequeue(rt);
            if (strcmp(ev->name, "message") == 0) {
                JSValue resp = JS_DupValue(ctx, ev->args[0]);
                free(ev->name);
                return resp;
            }
            // Other events should be queued back or handled?
            // Requirement 1.2 says block coroutine via __alloy_await
        }
    }

    return JS_UNDEFINED;
}

JSValue alloy_runtime_spawn(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_ThrowTypeError(ctx, "Missing command");
    JSCStringBuf buf;
    const char *cmd = JS_ToCString(ctx, argv[0], &buf);
    if (!cmd) return JS_EXCEPTION;

    JSValue res = JS_NewObject(ctx);
    pid_t pid = fork();
    if (pid == 0) { execlp(cmd, cmd, (char *)NULL); _exit(127); }
    else if (pid > 0) {
        JS_SetPropertyStr(ctx, res, "pid", JS_NewInt32(ctx, pid));
        return res;
    }
    else return JS_ThrowInternalError(ctx, "fork failed: %s", strerror(errno));
}

JSValue alloy_runtime_spawn_sync(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_ThrowTypeError(ctx, "Missing command");
    JSCStringBuf buf;
    const char *cmd = JS_ToCString(ctx, argv[0], &buf);
    if (!cmd) return JS_EXCEPTION;
    int status = system(cmd);
    JSValue res = JS_NewObject(ctx);
    JS_SetPropertyStr(ctx, res, "exitCode", JS_NewInt32(ctx, WEXITSTATUS(status)));
    return res;
}

JSValue alloy_runtime_secure_eval_internal(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) {
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

JSValue alloy_runtime_parse_validate(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) {
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

JSValue alloy_runtime_await_helper(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) {
    return JS_UNDEFINED;
}

JSValue alloy_runtime_deferred_helper(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) {
    return JS_UNDEFINED;
}

JSValue alloy_runtime_get_mmap(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) { return JS_UNDEFINED; }
JSValue alloy_runtime_read_mmap(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) { return JS_UNDEFINED; }

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
}

void alloy_drain_next_tick(alloy_runtime_t *rt) {
    while (rt->next_tick_count > 0) {
        JSValue func = rt->next_tick_queue[0];
        for (int i = 0; i < rt->next_tick_count - 1; i++) rt->next_tick_queue[i] = rt->next_tick_queue[i+1];
        rt->next_tick_count--;
        mprotect(rt->mem_buf, rt->js_heap_size, PROT_READ | PROT_WRITE);
        JSValue res = JS_Call(rt->mqjs_ctx, func, JS_UNDEFINED, 0, NULL);
        mprotect(rt->mem_buf, rt->js_heap_size, PROT_NONE);
        JS_FreeValue(rt->mqjs_ctx, func);
        JS_FreeValue(rt->mqjs_ctx, res);
    }
}

JSValue alloy_runtime_on(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) {
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

JSValue alloy_runtime_off(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) { return JS_UNDEFINED; }
JSValue alloy_runtime_emit(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) {
    alloy_runtime_t *rt = JS_GetContextOpaque(ctx);
    const char *name = JS_ToCString(ctx, argv[0], NULL);
    alloy_event_emit(rt, name, argv[1]);
    JS_FreeCString(ctx, name);
    return JS_UNDEFINED;
}

JSValue alloy_runtime_next_tick(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) {
    alloy_runtime_t *rt = JS_GetContextOpaque(ctx);
    if (rt->next_tick_count < 256) {
        rt->next_tick_queue[rt->next_tick_count++] = JS_DupValue(ctx, argv[0]);
    }
    return JS_UNDEFINED;
}

JSValue alloy_runtime_atob(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) { return JS_UNDEFINED; }
JSValue alloy_runtime_btoa(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) { return JS_UNDEFINED; }
JSValue alloy_runtime_structured_clone(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) { return JS_UNDEFINED; }
JSValue alloy_runtime_queue_microtask(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) { return JS_UNDEFINED; }
JSValue alloy_runtime_set_interval(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) { return JS_UNDEFINED; }
JSValue alloy_runtime_clear_interval(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) { return JS_UNDEFINED; }
JSValue alloy_runtime_clear_timeout(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) { return JS_UNDEFINED; }
JSValue alloy_runtime_alert(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) { return JS_UNDEFINED; }
JSValue alloy_runtime_confirm(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) { return JS_UNDEFINED; }
JSValue alloy_runtime_prompt(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) { return JS_UNDEFINED; }
JSValue alloy_runtime_report_error(JSContext *ctx, JSValue this_val, int argc, JSValue *argv) { return JS_UNDEFINED; }

alloy_runtime_t *alloy_engine_create(void) {
    alloy_runtime_t *rt = calloc(1, sizeof(alloy_runtime_t));
    if (!rt) return NULL;

    rt->js_heap_size = 16 << 20;
    rt->mem_buf = mmap(NULL, rt->js_heap_size, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    rt->mqjs_ctx = JS_NewContext(rt->mem_buf, rt->js_heap_size, &js_stdlib);
    JS_SetContextOpaque(rt->mqjs_ctx, rt);
    mprotect(rt->mem_buf, rt->js_heap_size, PROT_NONE);

    rt->webview = webview_create(0, NULL);
    webview_set_size(rt->webview, 1, 1, WEBVIEW_HINT_MIN);

    RAND_bytes(rt->ipc_state.session_key, 32);
    mlock(rt->ipc_state.session_key, 32);
    rt->ipc_state.next_outbound_seq = 1;
    rt->ipc_state.last_inbound_seq = 0;

    JSContext *ctx = rt->mqjs_ctx;
    mprotect(rt->mem_buf, rt->js_heap_size, PROT_READ | PROT_WRITE);
    JSValue global_obj = JS_GetGlobalObject(ctx);

    for (int i = 0; ALLOY_BROWSER_API_TABLE[i].js_name != NULL; i++) {
        alloy_mqjs_bind_func(ctx, ALLOY_BROWSER_API_TABLE[i].js_name, (JSCFunction *)alloy_browser_proxy_dispatch, (void *)&ALLOY_BROWSER_API_TABLE[i]);
    }

    JS_SetPropertyStr(ctx, global_obj, "window", JS_DupValue(ctx, global_obj));
    JS_SetPropertyStr(ctx, global_obj, "globalThis", JS_DupValue(ctx, global_obj));
    JS_SetPropertyStr(ctx, global_obj, "self", JS_DupValue(ctx, global_obj));
    JS_SetPropertyStr(ctx, global_obj, "global", JS_DupValue(ctx, global_obj));

    JSValue alloy_obj = JS_NewObject(ctx);
    JS_SetPropertyStr(ctx, alloy_obj, "spawn", JS_NewCFunction(ctx, alloy_runtime_spawn, "spawn", 3));
    JS_SetPropertyStr(ctx, alloy_obj, "spawnSync", JS_NewCFunction(ctx, alloy_runtime_spawn_sync, "spawnSync", 3));
    JS_SetPropertyStr(ctx, alloy_obj, "secureEval", JS_NewCFunction(ctx, alloy_runtime_secure_eval_internal, "secureEval", 1));
    JS_SetPropertyStr(ctx, alloy_obj, "getMmap", JS_NewCFunction(ctx, alloy_runtime_get_mmap, "getMmap", 0));
    JS_SetPropertyStr(ctx, alloy_obj, "readMmap", JS_NewCFunction(ctx, alloy_runtime_read_mmap, "readMmap", 1));
    JS_SetPropertyStr(ctx, alloy_obj, "on", JS_NewCFunction(ctx, alloy_runtime_on, "on", 2));
    JS_SetPropertyStr(ctx, alloy_obj, "off", JS_NewCFunction(ctx, alloy_runtime_off, "off", 2));
    JS_SetPropertyStr(ctx, alloy_obj, "emit", JS_NewCFunction(ctx, alloy_runtime_emit, "emit", 2));
    JS_SetPropertyStr(ctx, alloy_obj, "nextTick", JS_NewCFunction(ctx, alloy_runtime_next_tick, "nextTick", 1));
    JS_SetPropertyStr(ctx, global_obj, "Alloy", alloy_obj);

    JSValue internal_obj = JS_NewObject(ctx);
    JS_SetPropertyStr(ctx, internal_obj, "parse", JS_NewCFunction(ctx, alloy_runtime_parse_validate, "parse", 1));
    JS_SetPropertyStr(ctx, internal_obj, "__alloy_await", JS_NewCFunction(ctx, alloy_runtime_await_helper, "__alloy_await", 1));
    JS_SetPropertyStr(ctx, internal_obj, "__alloy_deferred", JS_NewCFunction(ctx, alloy_runtime_deferred_helper, "__alloy_deferred", 1));
    JS_SetPropertyStr(ctx, global_obj, "AlloyInternal", internal_obj);

    JS_SetPropertyStr(ctx, global_obj, "atob", JS_NewCFunction(ctx, alloy_runtime_atob, "atob", 1));
    JS_SetPropertyStr(ctx, global_obj, "btoa", JS_NewCFunction(ctx, alloy_runtime_btoa, "btoa", 1));
    JS_SetPropertyStr(ctx, global_obj, "structuredClone", JS_NewCFunction(ctx, alloy_runtime_structured_clone, "structuredClone", 1));
    JS_SetPropertyStr(ctx, global_obj, "queueMicrotask", JS_NewCFunction(ctx, alloy_runtime_queue_microtask, "queueMicrotask", 1));
    JS_SetPropertyStr(ctx, global_obj, "setInterval", JS_NewCFunction(ctx, alloy_runtime_set_interval, "setInterval", 2));
    JS_SetPropertyStr(ctx, global_obj, "clearInterval", JS_NewCFunction(ctx, alloy_runtime_clear_interval, "clearInterval", 1));
    JS_SetPropertyStr(ctx, global_obj, "clearTimeout", JS_NewCFunction(ctx, alloy_runtime_clear_timeout, "clearTimeout", 1));
    JS_SetPropertyStr(ctx, global_obj, "alert", JS_NewCFunction(ctx, alloy_runtime_alert, "alert", 1));
    JS_SetPropertyStr(ctx, global_obj, "confirm", JS_NewCFunction(ctx, alloy_runtime_confirm, "confirm", 1));
    JS_SetPropertyStr(ctx, global_obj, "prompt", JS_NewCFunction(ctx, alloy_runtime_prompt, "prompt", 2));
    JS_SetPropertyStr(ctx, global_obj, "reportError", JS_NewCFunction(ctx, alloy_runtime_report_error, "reportError", 1));

    JS_FreeValue(ctx, global_obj);
    mprotect(rt->mem_buf, rt->js_heap_size, PROT_NONE);

    return rt;
}

void alloy_engine_destroy(alloy_runtime_t *rt) {
    if (!rt) return;
    webview_destroy(rt->webview);
    JS_FreeContext(rt->mqjs_ctx);
    explicit_bzero(rt->ipc_state.session_key, 32);
    munmap(rt->mem_buf, rt->js_heap_size);
    free(rt);
}

void alloy_run(alloy_runtime_t *rt) {
    while (!rt->shutdown) {
        alloy_poll_io(rt);
        while (rt->event_queue.count > 0) {
            AlloyEvent *ev = alloy_event_dequeue(rt);
            mprotect(rt->mem_buf, rt->js_heap_size, PROT_READ | PROT_WRITE);
            JSValue res = JS_Call(rt->mqjs_ctx, ev->listener, JS_UNDEFINED, ev->argc, ev->args);
            mprotect(rt->mem_buf, rt->js_heap_size, PROT_NONE);
            if (JS_IsException(res)) {
                alloy_event_emit(rt, "error", JS_GetException(rt->mqjs_ctx));
            }
            JS_FreeValue(rt->mqjs_ctx, res);
            JS_FreeValue(rt->mqjs_ctx, ev->listener);
            for (int i = 0; i < ev->argc; i++) JS_FreeValue(rt->mqjs_ctx, ev->args[i]);
            free(ev->name);
        }
        alloy_drain_next_tick(rt);
        if (rt->event_queue.count == 0) {
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

#include "engine.h"
#include "plugin.h"
#include "mqjs_stdlib.h"
#include "ipc.h"
#include "browser_api_table.h"
#include "backend_adapters/mqjs_bridge.h"
#include "blake3/blake3.h"
#include "alloy/config.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <openssl/rand.h>
#include <sys/mman.h>
#include <unistd.h>

/* Forward declarations */
void alloy_stdlib_init(JSContext *ctx);

/* Task 3.2 & 3.5: Implement alloy_engine_create() with fixed mem_buf (8MB) */
alloy_runtime_t *alloy_engine_create(void) {
    alloy_runtime_t *rt = calloc(1, sizeof(alloy_runtime_t));
    if (!rt) return NULL;

    RAND_bytes(rt->ipc_state.session_key, ALLOY_IPC_KEY_SIZE);
    mlock(rt->ipc_state.session_key, ALLOY_IPC_KEY_SIZE);
    rt->ipc_state.next_outbound_seq = 1;
    rt->ipc_state.last_inbound_seq = 0;

    /* Task 3.5: Add MQJS_MEM_SIZE constant (8 * 1024 * 1024) */
    rt->js_heap_size = MQJS_MEM_SIZE;
    rt->js_heap = mmap(NULL, rt->js_heap_size, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    rt->js_heap_prot = PROT_READ | PROT_WRITE;

    /* Use minimal stdlib, register functions in alloy_stdlib_init */
    rt->mqjs_ctx = JS_NewContext(rt->js_heap, rt->js_heap_size, &js_stdlib);
    JS_SetContextOpaque(rt->mqjs_ctx, rt);

    rt->webview = webview_create(0, NULL);
    webview_bind(rt->webview, "__alloy_ipc_send", alloy_webview_dispatch, rt);
    webview_set_size(rt->webview, 1, 1, WEBVIEW_HINT_MIN);
    webview_init(rt->webview, "window.AlloySessionKey = '...';");

    /* Task 4.7: Register all stdlib functions */
    alloy_stdlib_init(rt->mqjs_ctx);

    /* Task 5.13: Keep ONLY minimal engine.c bootstrap */

    /* Load core plugins */
    extern RuntimePlugin alloy_config_plugin;
    extern RuntimePlugin alloy_web_plugin;
    extern RuntimePlugin alloy_shell_plugin;

    alloy_config_plugin.on_init(rt);
    alloy_web_plugin.on_init(rt);
    alloy_shell_plugin.on_init(rt);

    return rt;
}

void alloy_engine_destroy(alloy_runtime_t *rt) {
    if (!rt) return;
    if (rt->webview) webview_destroy(rt->webview);
    if (rt->mqjs_ctx) JS_FreeContext(rt->mqjs_ctx);
    if (rt->js_heap) munmap(rt->js_heap, rt->js_heap_size);
    explicit_bzero(rt->ipc_state.session_key, ALLOY_IPC_KEY_SIZE);
    free(rt);
}

JSValue alloy_secure_eval(alloy_runtime_t *rt, const char *script) {
    return JS_Eval(rt->mqjs_ctx, script, strlen(script), "<secureEval>", JS_EVAL_RETVAL);
}

void alloy_run(alloy_runtime_t *rt) {
    while (!rt->shutdown) {
        webview_run(rt->webview);
        while (rt->event_queue.count > 0) {
            AlloyEvent *ev = &rt->event_queue.events[rt->event_queue.head];
            mprotect(rt->js_heap, rt->js_heap_size, PROT_READ | PROT_WRITE);
            JSValue res = JS_Call(rt->mqjs_ctx, ev->listener, JS_UNDEFINED, ev->argc, ev->args);
            mprotect(rt->js_heap, rt->js_heap_size, PROT_NONE);
            JS_FreeValue(rt->mqjs_ctx, res);
            JS_FreeValue(rt->mqjs_ctx, ev->listener);
            rt->event_queue.head = (rt->event_queue.head + 1) % 4096;
            rt->event_queue.count--;
        }
        usleep(1000);
    }
}

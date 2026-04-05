#ifndef ALLOYSCRIPT_ENGINE_H
#define ALLOYSCRIPT_ENGINE_H

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>
#include "mquickjs.h"
#include "../../packages/webview/core/include/webview/webview.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef enum {
    ALLOY_TARGET_NATIVE = 0,
    ALLOY_TARGET_BROWSER = 1,
} AlloyTarget;

typedef struct {
    uint8_t session_key[32];
    uint64_t next_outbound_seq;
    uint64_t last_inbound_seq;
} AlloyIPCState;

#define ALLOY_EVENT_QUEUE_CAPACITY 4096

typedef struct {
    uint32_t    type;
    char       *name;
    JSValue     listener;
    JSValue     args[8];
    uint8_t     argc;
} AlloyEvent;

typedef struct {
    AlloyEvent  events[ALLOY_EVENT_QUEUE_CAPACITY];
    uint32_t    head;
    uint32_t    tail;
    uint32_t    count;
} AlloyEventQueue;

typedef struct {
    const char *event_name;
    JSValue     listeners[8];
    uint8_t     count;
} AlloyListenerEntry;

#define ALLOY_MMAP_MAX_REGIONS 128

typedef enum {
    ALLOY_REGION_CODE,
    ALLOY_REGION_RODATA,
    ALLOY_REGION_DATA,
    ALLOY_REGION_STACK,
    ALLOY_REGION_HEAP,
    ALLOY_REGION_JS_ARENA,
    ALLOY_REGION_VDSO,
    ALLOY_REGION_ANON,
    ALLOY_REGION_UNKNOWN,
} AlloyRegionKind;

typedef struct {
    char             name[64];
    AlloyRegionKind  kind;
    uintptr_t        start;
    size_t           size;
    char             perms[8];   // "r-x\0", "rw-\0", etc.
    bool             readable;
    bool             writable;
    bool             executable;
    bool             stable;
    char             file[256];  // backing file path or ""
} AlloyMmapRegion;

typedef struct {
    AlloyMmapRegion  regions[ALLOY_MMAP_MAX_REGIONS];
    int              count;
    uintptr_t        aslr_offset;
    uint8_t          blake3_text[32];        // zeroed if .text not found
    uint8_t          blake3_rodata[32];      // zeroed if .rodata not found
    uint8_t          blake3_text_rodata[32]; // zeroed if neither found
    bool             js_arena_protected;
} AlloyMmapSnapshot;

#include "js_engine.h"

typedef struct alloy_runtime {
    AlloyJSEngine    *mqjs_engine;     // trusted primary — MQJS vtable
    AlloyJSEngine    *webview_engine;  // untrusted secondary — WebView vtable
    JSContext *mqjs_ctx; // Still useful for some internal MQJS calls
    webview_t webview;   // Still useful for internal webview calls
    void *mem_buf;
    size_t js_heap_size;
    int js_heap_prot;
    AlloyTarget target;
    AlloyIPCState ipc_state;
    AlloyEventQueue event_queue;
    AlloyListenerEntry listener_registry[32];
    JSValue next_tick_queue[256];
    uint8_t next_tick_count;
    bool shutdown;
} alloy_runtime_t;

typedef void (*alloy_bind_fn_t)(const char *id, const char *req, void *arg);

alloy_runtime_t *alloy_engine_create(void);
void alloy_engine_free(alloy_runtime_t *rt);
void alloy_run(alloy_runtime_t *rt);

void alloy_bind_global(alloy_runtime_t *rt, const char *name, alloy_bind_fn_t fn, void *arg);
void alloy_return(alloy_runtime_t *rt, const char *id, int status, const char *result);

JSValue alloy_secure_eval(alloy_runtime_t *rt, const char *script);

/* Event Loop */
void alloy_event_emit(alloy_runtime_t *rt, const char *name, JSValue arg);
AlloyEvent *alloy_event_dequeue(alloy_runtime_t *rt);
void alloy_poll_io(alloy_runtime_t *rt);
void alloy_drain_next_tick(alloy_runtime_t *rt);

/* Native Bindings */
JSValue alloy_runtime_spawn(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue alloy_runtime_spawn_sync(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue alloy_runtime_secure_eval_internal(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue alloy_runtime_get_mmap(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue alloy_runtime_read_mmap(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue alloy_runtime_on(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue alloy_runtime_off(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue alloy_runtime_emit(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue alloy_runtime_next_tick(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);

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

JSValue alloy_runtime_parse_validate(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue alloy_runtime_await_helper(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue alloy_runtime_deferred_helper(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);

#ifdef __cplusplus
}
#endif

#endif // ALLOYSCRIPT_ENGINE_H

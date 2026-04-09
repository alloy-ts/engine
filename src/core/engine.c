#include <stddef.h>
#include <stdlib.h>
#include <string.h>
#include "alloy/alloy.h"
#include "alloy/config.h"

struct AlloyRuntime { JSContext *ctx; uint8_t *mem_buf; };

extern const JSSTDLibraryDef alloyscript_stdlib;
extern void config_plugin_on_init(JSContext *ctx);
#ifdef ALLOY_ENABLE_WEB
extern void web_plugin_on_init(JSContext *ctx);
extern void shell_plugin_on_init(JSContext *ctx);
#endif

AlloyRuntime* alloy_engine_create(void) {
    AlloyRuntime *rt = malloc(sizeof(AlloyRuntime));
    if (!rt) return NULL;
    rt->mem_buf = malloc(MQJS_MEM_SIZE);
    if (!rt->mem_buf) { free(rt); return NULL; }
    rt->ctx = JS_NewContext(rt->mem_buf, MQJS_MEM_SIZE, &alloyscript_stdlib);
    if (!rt->ctx) { free(rt->mem_buf); free(rt); return NULL; }
    JS_SetContextOpaque(rt->ctx, rt);
    config_plugin_on_init(rt->ctx);
#ifdef ALLOY_ENABLE_WEB
    web_plugin_on_init(rt->ctx);
    shell_plugin_on_init(rt->ctx);
#endif
    return rt;
}

JSContext* get_rt_ctx(AlloyRuntime* rt) { return rt->ctx; }

void alloy_engine_destroy(AlloyRuntime *rt) {
    if (!rt) return;
    if (rt->ctx) JS_FreeContext(rt->ctx);
    if (rt->mem_buf) free(rt->mem_buf);
    free(rt);
}

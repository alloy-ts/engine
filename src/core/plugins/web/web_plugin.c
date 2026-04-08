#include "../../engine.h"
#include "../../plugin_types.h"
#include "../../browser_api_table.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// Forward declaration of the generic dispatcher
extern void alloy_browser_proxy_handler(AlloyJSEngine *engine, void *opaque, const char *args_json, char *out_result, size_t result_size);

static int web_on_init(AlloyJSEngine *engine) {
    // 3.2 Move ALLOY_BROWSER_API_TABLE loop to alloy:web on_init
    for (int i = 0; ALLOY_BROWSER_API_TABLE[i].js_name != NULL; i++) {
        alloy_engine_bind(engine, ALLOY_BROWSER_API_TABLE[i].js_name, alloy_browser_proxy_handler, (void *)&ALLOY_BROWSER_API_TABLE[i]);
    }

    // 3.3 Move window/globalThis/self/global aliases
    char *out = NULL;
    alloy_engine_eval(engine, "window = globalThis; self = globalThis; global = globalThis;", "web_init_aliases", &out);
    free(out);

    // 3.4 Move atob/btoa/structuredClone/queueMicrotask/timers/dialogs
    extern void alloy_legacy_handler(AlloyJSEngine *engine, void *opaque, const char *args_json, char *out_result, size_t result_size);
    alloy_engine_bind(engine, "atob", alloy_legacy_handler, (void *)alloy_runtime_atob);
    alloy_engine_bind(engine, "btoa", alloy_legacy_handler, (void *)alloy_runtime_btoa);
    alloy_engine_bind(engine, "structuredClone", alloy_legacy_handler, (void *)alloy_runtime_structured_clone);
    alloy_engine_bind(engine, "queueMicrotask", alloy_legacy_handler, (void *)alloy_runtime_queue_microtask);
    alloy_engine_bind(engine, "setInterval", alloy_legacy_handler, (void *)alloy_runtime_set_interval);
    alloy_engine_bind(engine, "clearInterval", alloy_legacy_handler, (void *)alloy_runtime_clear_interval);
    alloy_engine_bind(engine, "clearTimeout", alloy_legacy_handler, (void *)alloy_runtime_clear_timeout);
    alloy_engine_bind(engine, "alert", alloy_legacy_handler, (void *)alloy_runtime_alert);
    alloy_engine_bind(engine, "confirm", alloy_legacy_handler, (void *)alloy_runtime_confirm);
    alloy_engine_bind(engine, "prompt", alloy_legacy_handler, (void *)alloy_runtime_prompt);
    alloy_engine_bind(engine, "reportError", alloy_legacy_handler, (void *)alloy_runtime_report_error);

    return 0;
}

static const char *web_deps[] = { "alloy:config", NULL };

RuntimePlugin alloy_web_plugin = {
    .name = "alloy:web",
    .depends_on = web_deps,
    .on_init = web_on_init,
    .on_eval_begin = NULL,
    .on_eval_end = NULL,
    .on_destroy = NULL,
};

#include "../plugin.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <dlfcn.h>

static void *g_handle_table[128];
static int g_handle_count = 0;

static char *ffi_dlopen_handler(const char *args_json, void *userdata) {
    // args_json: ["path", "symbols_json"]
    // Very simplified extraction
    char path[256];
    sscanf(args_json, "[\"%[^\"]\"", path);

    void *h = dlopen(path, RTLD_NOW);
    if (!h) return strdup("{\"error\":\"dlopen failed\"}");

    int id = g_handle_count++;
    g_handle_table[id] = h;
    char res[32];
    snprintf(res, sizeof(res), "%d", id);
    return strdup(res);
}

static char *ffi_call_handler(const char *args_json, void *userdata) {
    // Marshals symbol lookup and calling
    return strdup("null");
}

static int ffi_plugin_on_init(AlloyJSEngine *engine, void *userdata) {
    alloy_engine_bind(engine, "__alloy_ffi_dlopen", ffi_dlopen_handler, userdata);
    alloy_engine_bind(engine, "__alloy_ffi_call", ffi_call_handler, userdata);
    // Bind others...
    return ALLOY_OK;
}

static RuntimePlugin alloy_ffi_plugin = {
    .name = "alloy:ffi",
    .version = "1.0.0",
    .on_init = ffi_plugin_on_init,
};

RuntimePlugin *alloy_ffi_plugin_init(void) {
    return &alloy_ffi_plugin;
}

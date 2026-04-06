/**
 * AlloyScript — Engine Initialisation + Plugin Load Pipeline
 *
 * Hook execution responsibility — everything lives here, nothing in main_host:
 *
 *   on_init       → fired by load_plugin() during alloy_engine_create_with_config()
 *   on_eval_begin → fired by alloy_runtime_eval() before every user script eval
 *   on_eval_end   → fired by alloy_runtime_eval() after every user script eval
 *   on_destroy    → fired by unload_plugins() during alloy_engine_destroy_runtime()
 *
 * main_host.c calls only alloy_engine_create_with_config(), alloy_runtime_eval(),
 * and alloy_engine_destroy_runtime(). It never touches plugin hooks directly.
 */
#include "../../include/alloy/js_engine.h"
#include "../../include/alloy/plugin.h"
#include "../../include/alloy/capability.h"
#include "../../include/alloy/config.h"
#include "../../include/alloy/mqjs_compat.h"
#include "../../packages/mquickjs/mquickjs.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>

AlloyJSEngine *alloy_mqjs_adapter_create(JSContext *ctx);
RuntimePlugin *alloy_config_plugin_create(AlloyRuntimeConfig cfg);
RuntimePlugin *alloy_web_runtime_create(AlloyCapStore *cap_store);
/** Returns true if the plugin was disabled by config after on_init ran. */
bool alloy_web_runtime_is_disabled(RuntimePlugin *p);
const JSSTDLibraryDef *alloy_get_stdlib(void);

extern const char __alloy_runtime_bootstrap_js[];
extern const char __alloy_web_shims_js[];

/* ── AlloyRuntime ────────────────────────────────────────────────────────── */
#define ALLOY_MAX_PLUGINS 32

typedef struct {
    AlloyJSEngine      *engine;
    AlloyCapStore      *cap_store;
    AlloyRuntimeConfig  config;
    RuntimePlugin      *plugins[ALLOY_MAX_PLUGINS];
    int                 plugin_count;
    void               *js_heap;
    size_t              js_heap_size;
} AlloyRuntime;

static AlloyRuntime *g_runtime = NULL;

/* ── Plugin helpers ─────────────────────────────────────────────────────── */
static int load_plugin(AlloyRuntime *rt, RuntimePlugin *plugin) {
    if (rt->plugin_count >= ALLOY_MAX_PLUGINS) return -1;
    if (plugin->on_init) {
        int r = plugin->on_init(plugin, rt->engine);
        if (r != ALLOY_OK) {
            fprintf(stderr, "[alloy] plugin '%s' on_init failed: %d\n",
                    plugin->name, r);
            return r;
        }
    }
    rt->plugins[rt->plugin_count++] = plugin;

    /* Register in Alloy.plugins[name] so JS can see it.
     * Sets Alloy.plugins["alloy:web"] = { name, version } at minimum.
     * The plugin's JS entry point (if any) can enrich this later. */
    char script[256];
    snprintf(script, sizeof(script),
        "if(typeof Alloy!=='undefined'&&Alloy.plugins){"
        "  Alloy.plugins[\"%s\"]=Alloy.plugins[\"%s\"]||"
        "  {name:\"%s\",version:\"%s\"};"
        "}",
        plugin->name, plugin->name,
        plugin->name, plugin->version ? plugin->version : "0.0.0");
    char *r = alloy_engine_eval(rt->engine, script, "<plugin_register>");
    free(r);

    return ALLOY_OK;
}

static void unload_plugins(AlloyRuntime *rt) {
    for (int i = rt->plugin_count - 1; i >= 0; i--) {
        RuntimePlugin *p = rt->plugins[i];
        if (p && p->on_destroy) p->on_destroy(p, rt->engine);
        free(p);
    }
    rt->plugin_count = 0;
}

/* ── alloy_engine_create_with_config ────────────────────────────────────── */
AlloyRuntime *alloy_engine_create_with_config(AlloyRuntimeConfig cfg) {
    AlloyRuntime *rt = calloc(1, sizeof(AlloyRuntime));
    if (!rt) return NULL;
    rt->config = cfg;

    /* 1. mmap JS heap */
    rt->js_heap_size = cfg.js_heap_size > 0 ? cfg.js_heap_size : 16 * 1024 * 1024;
    rt->js_heap = mmap(NULL, rt->js_heap_size,
                       PROT_READ | PROT_WRITE,
                       MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    if (rt->js_heap == MAP_FAILED) {
        fprintf(stderr, "[alloy] mmap failed\n");
        free(rt);
        return NULL;
    }

    /* 2. MicroQuickJS context */
    JSContext *ctx = JS_NewContext(rt->js_heap, rt->js_heap_size, alloy_get_stdlib());
    if (!ctx) {
        fprintf(stderr, "[alloy] JS_NewContext failed\n");
        munmap(rt->js_heap, rt->js_heap_size);
        free(rt);
        return NULL;
    }

    /* 3. AlloyJSEngine vtable */
    rt->engine = alloy_mqjs_adapter_create(ctx);
    if (!rt->engine) {
        JS_FreeContext(ctx);
        munmap(rt->js_heap, rt->js_heap_size);
        free(rt);
        return NULL;
    }

    /* 4. Bootstrap */
    char *br = alloy_engine_eval(rt->engine, __alloy_runtime_bootstrap_js, "<bootstrap>");
    if (br) {
        if (strstr(br, "\"error\""))
            fprintf(stderr, "[alloy] bootstrap error: %s\n", br);
        free(br);
    }

    /* 5. Capability store */
    rt->cap_store = alloy_cap_store_create();
    if (!rt->cap_store) {
        fprintf(stderr, "[alloy] cap store creation failed\n");
        alloy_engine_destroy(rt->engine);
        munmap(rt->js_heap, rt->js_heap_size);
        free(rt);
        return NULL;
    }

    /* 6. Load plugins in dependency order:
     *      alloy:config  — must run first; serialises AlloyRuntimeConfig into
     *                      Alloy.config and registers __alloy_use_config /
     *                      __alloy_config_set bridges.
     *      alloy:web     — reads noWebAPI via __alloy_use_config; skips sidecar
     *                      spawn if noWebAPI=true. Only compiled in when
     *                      ALLOY_ENABLE_WEB is defined.
     */
    RuntimePlugin *config_plugin = alloy_config_plugin_create(cfg);
    if (config_plugin) {
        if (load_plugin(rt, config_plugin) != ALLOY_OK)
            fprintf(stderr, "[alloy] failed to load alloy:config plugin\n");
    }

#ifdef ALLOY_ENABLE_WEB
    {
        RuntimePlugin *web = alloy_web_runtime_create(rt->cap_store);
        if (web) {
            if (load_plugin(rt, web) != ALLOY_OK) {
                fprintf(stderr, "[alloy] failed to load alloy:web plugin\n");
            } else if (!alloy_web_runtime_is_disabled(web)) {
                /* Only inject shims when the sidecar is actually enabled */
                char *sr = alloy_engine_eval(rt->engine,
                                             __alloy_web_shims_js, "<web_shims>");
                if (sr) {
                    if (strstr(sr, "\"error\""))
                        fprintf(stderr, "[alloy] web shims error: %s\n", sr);
                    free(sr);
                }
            }
        }
    }
#endif

    g_runtime = rt;
    return rt;
}

/* Convenience: default config */
AlloyRuntime *alloy_engine_create(void) {
    return alloy_engine_create_with_config(alloy_config_defaults());
}

/* ── Teardown ────────────────────────────────────────────────────────────── */
void alloy_engine_destroy_runtime(AlloyRuntime *rt) {
    if (!rt) return;
    unload_plugins(rt);
    alloy_cap_store_destroy(rt->cap_store);
    alloy_engine_destroy(rt->engine);
    if (rt->js_heap) munmap(rt->js_heap, rt->js_heap_size);
    free(rt);
    if (g_runtime == rt) g_runtime = NULL;
}

AlloyJSEngine *alloy_engine_get(AlloyRuntime *rt) {
    return rt ? rt->engine : NULL;
}

AlloyCapStore *alloy_engine_get_cap_store(AlloyRuntime *rt) {
    return rt ? rt->cap_store : NULL;
}

int alloy_engine_plugin_count(AlloyRuntime *rt) {
    return rt ? rt->plugin_count : 0;
}

const RuntimePlugin *alloy_engine_get_plugin(AlloyRuntime *rt, int i) {
    if (!rt || i < 0 || i >= rt->plugin_count) return NULL;
    return rt->plugins[i];
}

const RuntimePlugin *alloy_engine_find_plugin(AlloyRuntime *rt, const char *name) {
    if (!rt || !name) return NULL;
    for (int i = 0; i < rt->plugin_count; i++) {
        if (rt->plugins[i] && strcmp(rt->plugins[i]->name, name) == 0)
            return rt->plugins[i];
    }
    return NULL;
}

void alloy_engine_notify_eval_begin(AlloyRuntime *rt, const char *source) {
    if (!rt) return;
    for (int i = 0; i < rt->plugin_count; i++) {
        RuntimePlugin *p = rt->plugins[i];
        if (p && p->on_eval_begin) p->on_eval_begin(p, rt->engine, source);
    }
}

void alloy_engine_notify_eval_end(AlloyRuntime *rt, const char *result_json) {
    if (!rt) return;
    for (int i = 0; i < rt->plugin_count; i++) {
        RuntimePlugin *p = rt->plugins[i];
        if (p && p->on_eval_end) p->on_eval_end(p, rt->engine, result_json);
    }
}

/**
 * alloy_runtime_eval — the single entry point for evaluating user scripts.
 *
 * Fires on_eval_begin on all plugins, evaluates the script via the engine
 * vtable, fires on_eval_end on all plugins, then returns the result.
 *
 * main_host.c calls this instead of calling the three steps separately.
 * Callers must free() the returned string.
 */
char *alloy_runtime_eval(AlloyRuntime *rt, const char *source,
                          const char *filename) {
    if (!rt || !source) return NULL;

    /* Fire on_eval_begin on every loaded plugin */
    for (int i = 0; i < rt->plugin_count; i++) {
        RuntimePlugin *p = rt->plugins[i];
        if (p && p->on_eval_begin) p->on_eval_begin(p, rt->engine, source);
    }

    /* Evaluate via the engine vtable */
    char *result = alloy_engine_eval(rt->engine, source,
                                      filename ? filename : "<eval>");

    /* Fire on_eval_end on every loaded plugin */
    const char *result_for_hooks = result ? result : "null";
    for (int i = 0; i < rt->plugin_count; i++) {
        RuntimePlugin *p = rt->plugins[i];
        if (p && p->on_eval_end) p->on_eval_end(p, rt->engine, result_for_hooks);
    }

    return result;
}

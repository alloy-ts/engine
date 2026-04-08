#include "engine.h"
#include "js_engine.h"
#include "plugin.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

extern AlloyJSEngine *alloy_mqjs_adapter_create(JSContext *ctx);
extern AlloyJSEngine *alloy_webview_adapter_create(webview_t w);

// Plugin Load Pipeline implementation

typedef struct {
    char *name;
    char **deps;
    int dep_count;
    int in_degree;
} TopoNode;

static int kahn_topo_sort(TopoNode *nodes, int n, int *order) {
    int count = 0;
    int *queue = malloc(n * sizeof(int));
    int head = 0, tail = 0;

    for (int i = 0; i < n; i++) {
        if (nodes[i].in_degree == 0) queue[tail++] = i;
    }

    while (head < tail) {
        int u = queue[head++];
        order[count++] = u;

        for (int v = 0; v < n; v++) {
            for (int d = 0; d < nodes[v].dep_count; d++) {
                if (strcmp(nodes[v].deps[d], nodes[u].name) == 0) {
                    nodes[v].in_degree--;
                    if (nodes[v].in_degree == 0) queue[tail++] = v;
                }
            }
        }
    }

    free(queue);
    return count == n ? 0 : ALLOY_ERR_CYCLE;
}

static char *load_bootstrap_js() {
    FILE *f = fopen("src/core/__alloy_runtime_bootstrap.js", "rb");
    if (!f) return NULL;
    fseek(f, 0, SEEK_END);
    size_t len = ftell(f);
    fseek(f, 0, SEEK_SET);
    char *buf = malloc(len + 1);
    fread(buf, 1, len, f);
    buf[len] = '\0';
    fclose(f);
    return buf;
}

extern RuntimePlugin *alloy_ffi_plugin_init(void);
extern RuntimePlugin *alloy_config_plugin_init(void);
extern RuntimePlugin *alloy_shell_plugin_init(void);

AlloyJSEngine *alloy_engine_create_v2(void *config, int type) {
    alloy_runtime_t *rt = (alloy_runtime_t *)config;
    AlloyJSEngine *adapter = NULL;
    if (type == 0) {
        // MQJS init
        adapter = alloy_mqjs_adapter_create(rt->mqjs_ctx);
    } else {
        // WebView init
        adapter = alloy_webview_adapter_create(rt->secondary_engine->engine);
    }

    if (!adapter) return NULL;

    // 1. Bootstrap
    char *bootstrap = load_bootstrap_js();
    char *res = alloy_engine_eval(adapter, bootstrap);
    free(res);
    free(bootstrap);

    // 2. Load Core Plugins in fixed order
    RuntimePlugin *core[] = {
        alloy_ffi_plugin_init(),
        alloy_config_plugin_init(),
        alloy_shell_plugin_init()
    };

    for (int i = 0; i < 3; i++) {
        core[i]->on_init(adapter, config);

        // Register in C registry if needed...

        // Evaluate JS entry for core plugin
        char entry_path[256];
        snprintf(entry_path, sizeof(entry_path), "src/core/plugins/%s/%s.js",
                 strchr(core[i]->name, ':') + 1, strchr(core[i]->name, ':') + 1);

        FILE *f = fopen(entry_path, "rb");
        if (f) {
            fseek(f, 0, SEEK_END);
            size_t len = ftell(f);
            fseek(f, 0, SEEK_SET);
            char *js = malloc(len + 1);
            fread(js, 1, len, f);
            js[len] = '\0';
            fclose(f);

            char *res = alloy_engine_eval(adapter, js);
            free(res);
            free(js);
        }
    }

    // 3. Post-load Start hook
    alloy_engine_call(adapter, "onStart", "[]");

    return adapter;
}

char *alloy_engine_dispatch_resolve(AlloyJSEngine *engine, const char *path, const char *importer) {
    char args[1024];
    snprintf(args, sizeof(args), "[\"%s\", \"%s\"]", path, importer);
    return alloy_engine_call(engine, "__alloy_dispatch_on_resolve", args);
}

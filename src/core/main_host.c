/*
 * AlloyScript — Core Host Entry Point
 *
 * Initialises the AlloyRuntime and evaluates the user script.
 * Plugins (alloy:config, alloy:web, etc.) inject their own behaviour
 * via the RuntimePlugin lifecycle hooks — this file knows nothing about
 * WebView, IPC, or any specific plugin.
 *
 * Two execution modes:
 *   1. Embedded (alloy compile):
 *      ALLOY_USER_SCRIPT_HEADER is defined at compile time.
 *      The binary runs the baked-in script with no external files.
 *
 *   2. File argument (development / alloy run):
 *      alloy_host <script.js>
 */
#include "../../include/alloy/runtime.h"
#include "../../include/alloy/js_engine.h"
#include "../../include/alloy/config.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* ── Embedded user script (set by alloy compile) ─────────────────────────── */
#ifdef ALLOY_USER_SCRIPT_HEADER
static const char g_user_script[] =
#  include ALLOY_USER_SCRIPT_HEADER
;
#  define ALLOY_HAS_EMBEDDED_SCRIPT 1
#else
#  define ALLOY_HAS_EMBEDDED_SCRIPT 0
static const char g_user_script[] = "";
#endif

/* ── File loader ─────────────────────────────────────────────────────────── */
static char *load_file(const char *path, size_t *out_len) {
    FILE *f = fopen(path, "rb");
    if (!f) { perror(path); return NULL; }
    fseek(f, 0, SEEK_END);
    long len = ftell(f);
    fseek(f, 0, SEEK_SET);
    char *buf = malloc((size_t)len + 1);
    if (!buf) { fclose(f); return NULL; }
    fread(buf, 1, (size_t)len, f);
    buf[len] = '\0';
    fclose(f);
    if (out_len) *out_len = (size_t)len;
    return buf;
}

/* ── Entry point ─────────────────────────────────────────────────────────── */
int main(int argc, char **argv) {
    const char *script      = NULL;
    char       *script_heap = NULL;
    const char *script_name = "<embedded>";

#if ALLOY_HAS_EMBEDDED_SCRIPT
    (void)argc; (void)argv;
    script = g_user_script;
#else
    if (argc < 2) {
        fprintf(stderr, "usage: alloy_host <script.js>\n");
        return 1;
    }
    script_name = argv[1];
    script_heap = load_file(script_name, NULL);
    if (!script_heap) return 1;
    script = script_heap;
#endif

    AlloyRuntimeConfig cfg = alloy_config_defaults();

    AlloyRuntime *rt = alloy_engine_create_with_config(cfg);
    if (!rt) {
        fprintf(stderr, "[alloy] failed to create runtime\n");
        free(script_heap);
        return 1;
    }

    AlloyJSEngine *eng = alloy_engine_get(rt);
    alloy_engine_notify_eval_begin(rt, script);
    char *result = alloy_engine_eval(eng, script, script_name);
    alloy_engine_notify_eval_end(rt, result ? result : "null");

    int exit_code = 0;
    if (result && strstr(result, "\"error\"")) {
        fprintf(stderr, "[alloy] %s\n", result);
        exit_code = 1;
    }

    free(result);
    free(script_heap);
    alloy_engine_destroy_runtime(rt);
    return exit_code;
}

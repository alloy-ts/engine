/**
 * AlloyScript — Runtime configuration (C side)
 *
 * Populated from alloyconfig.toml before alloy_engine_create() is called.
 * Mirrors the relevant fields of AlloyConfig (src/core/config/config.ts).
 */
#ifndef ALLOY_CONFIG_H
#define ALLOY_CONFIG_H

#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    /**
     * Disable the alloy:web sidecar entirely.
     * When true: no WebView process is spawned, no web API shims are injected,
     * and the binary has no WebKit/WebView2 dependency.
     * Equivalent to building with ALLOY_ENABLE_WEB=OFF but decided at runtime.
     */
    bool no_web_api;

    /** Log level: 0=debug, 1=warn, 2=error */
    int  log_level;

    /** JS heap size in bytes (default: 16 MiB) */
    size_t js_heap_size;
} AlloyRuntimeConfig;

/** Default configuration — all features enabled. */
static inline AlloyRuntimeConfig alloy_config_defaults(void) {
    AlloyRuntimeConfig c = {0};
    c.no_web_api   = false;
    c.log_level    = 1;   /* warn */
    c.js_heap_size = 16 * 1024 * 1024;
    return c;
}

#ifdef __cplusplus
}
#endif

#endif /* ALLOY_CONFIG_H */

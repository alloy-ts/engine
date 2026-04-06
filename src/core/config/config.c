/**
 * alloy:config plugin — C layer
 *
 * Responsibilities:
 *   1. On init: serialise AlloyRuntimeConfig → JSON → inject into
 *      Alloy.config inside MicroQuickJS so JS plugins can read it.
 *   2. Register __alloy_use_config(pluginName, schemaJson) → JSON
 *      Called by useConfig() in JS to register a plugin's config schema
 *      and get back the current values for that plugin's namespace.
 *   3. Register __alloy_config_set(pluginName, key, valueJson, persist) → JSON
 *      Called by ConfigAccessor.set() to update a config value at runtime.
 *
 * The config registry is the single source of truth. Plugins never read
 * alloyconfig.toml directly — they always go through useConfig().
 */
#include "../../js_engine.h"
#include "../../../../include/alloy/plugin.h"
#include "../../../../include/alloy/config.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* ── Config registry ────────────────────────────────────────────────────── */

#define MAX_PLUGIN_CONFIGS 32
#define MAX_CONFIG_KEYS    16

typedef struct {
    char key[64];
    char value_json[512];  /* current value as JSON */
} ConfigEntry;

typedef struct {
    char        plugin_name[64];
    ConfigEntry entries[MAX_CONFIG_KEYS];
    int         entry_count;
} PluginConfigSlice;

typedef struct {
    AlloyRuntimeConfig  runtime_cfg;
    PluginConfigSlice   slices[MAX_PLUGIN_CONFIGS];
    int                 slice_count;
    AlloyJSEngine      *engine;  /* back-reference for JS eval */
} ConfigPluginState;

/* ── Find or create a plugin config slice ───────────────────────────────── */

static PluginConfigSlice *get_or_create_slice(ConfigPluginState *st,
                                               const char *plugin_name) {
    for (int i = 0; i < st->slice_count; i++) {
        if (strcmp(st->slices[i].plugin_name, plugin_name) == 0)
            return &st->slices[i];
    }
    if (st->slice_count >= MAX_PLUGIN_CONFIGS) return NULL;
    PluginConfigSlice *s = &st->slices[st->slice_count++];
    memset(s, 0, sizeof(*s));
    strncpy(s->plugin_name, plugin_name, sizeof(s->plugin_name) - 1);
    return s;
}

static ConfigEntry *get_or_create_entry(PluginConfigSlice *slice,
                                         const char *key) {
    for (int i = 0; i < slice->entry_count; i++) {
        if (strcmp(slice->entries[i].key, key) == 0)
            return &slice->entries[i];
    }
    if (slice->entry_count >= MAX_CONFIG_KEYS) return NULL;
    ConfigEntry *e = &slice->entries[slice->entry_count++];
    memset(e, 0, sizeof(*e));
    strncpy(e->key, key, sizeof(e->key) - 1);
    return e;
}

/* ── Serialise AlloyRuntimeConfig to JSON for Alloy.config ─────────────── */

static char *runtime_config_to_json(const AlloyRuntimeConfig *cfg) {
    /* Build the top-level Alloy.config JSON object.
     * Plugin-specific slices are nested under Alloy.config.plugins.<name>. */
    char *buf = malloc(256);
    if (!buf) return NULL;
    snprintf(buf, 256,
        "{"
        "\"noWebAPI\":%s,"
        "\"logLevel\":%d,"
        "\"jsHeapSize\":%zu,"
        "\"plugins\":{}"
        "}",
        cfg->no_web_api ? "true" : "false",
        cfg->log_level,
        cfg->js_heap_size);
    return buf;
}

/* ── __alloy_use_config bridge ──────────────────────────────────────────── */
/*
 * Called from JS: __alloy_use_config(pluginName, schemaJson)
 * schemaJson: { "key": { "default": <value> }, ... }
 * Returns: { "values": { "key": <current_value_or_default>, ... } }
 *
 * This registers the schema and returns the current config values for
 * the plugin, merging defaults with any values from alloyconfig.toml.
 */
static char *use_config_handler(const char *name, const char *args_json,
                                  void *userdata) {
    (void)name;
    ConfigPluginState *st = (ConfigPluginState *)userdata;

    /* args_json is a JSON array: [pluginName, schemaJson] */
    /* Minimal extraction — find the two string arguments */
    const char *p = args_json;
    if (*p == '[') p++;
    if (*p == '"') p++;

    /* Extract plugin name */
    char plugin_name[64] = {0};
    const char *end = strchr(p, '"');
    if (!end) return strdup("{\"error\":\"bad args\"}");
    size_t nlen = (size_t)(end - p);
    if (nlen >= sizeof(plugin_name)) nlen = sizeof(plugin_name) - 1;
    memcpy(plugin_name, p, nlen);

    /* Find schema JSON (second argument) */
    p = end + 1;
    while (*p && *p != '{') p++;
    if (!*p) return strdup("{\"error\":\"missing schema\"}");
    const char *schema_start = p;

    /* Get or create the slice for this plugin */
    PluginConfigSlice *slice = get_or_create_slice(st, plugin_name);
    if (!slice) return strdup("{\"error\":\"config registry full\"}");

    /* Parse schema keys and populate defaults.
     * Schema format: { "key": { "default": <value> }, ... }
     * We scan for "key": { "default": <value> } patterns. */
    p = schema_start + 1;
    while (*p && *p != '}') {
        /* Find key name */
        while (*p && *p != '"' && *p != '}') p++;
        if (*p != '"') break;
        p++;
        const char *key_start = p;
        while (*p && *p != '"') p++;
        if (!*p) break;
        char key[64] = {0};
        size_t klen = (size_t)(p - key_start);
        if (klen >= sizeof(key)) klen = sizeof(key) - 1;
        memcpy(key, key_start, klen);
        p++;

        /* Find "default": <value> */
        const char *def_key = strstr(p, "\"default\"");
        if (!def_key) { while (*p && *p != ',') p++; continue; }
        def_key += 9; /* skip "default" */
        while (*def_key && (*def_key == ':' || *def_key == ' ')) def_key++;

        /* Extract the default value (up to , or }) */
        char default_val[256] = "null";
        const char *val_end = def_key;
        int depth = 0;
        while (*val_end) {
            if (*val_end == '{' || *val_end == '[') depth++;
            else if (*val_end == '}' || *val_end == ']') {
                if (depth == 0) break;
                depth--;
            } else if (*val_end == ',' && depth == 0) break;
            val_end++;
        }
        size_t vlen = (size_t)(val_end - def_key);
        if (vlen >= sizeof(default_val)) vlen = sizeof(default_val) - 1;
        memcpy(default_val, def_key, vlen);
        default_val[vlen] = '\0';
        /* Trim trailing whitespace */
        while (vlen > 0 && (default_val[vlen-1] == ' ' || default_val[vlen-1] == '\n'))
            default_val[--vlen] = '\0';

        /* Register entry with default if not already set */
        ConfigEntry *entry = get_or_create_entry(slice, key);
        if (entry && entry->value_json[0] == '\0') {
            strncpy(entry->value_json, default_val, sizeof(entry->value_json) - 1);
        }

        /* Advance past this schema entry */
        p = val_end;
        while (*p && *p != ',' && *p != '}') p++;
        if (*p == ',') p++;
    }

    /* Build response: { "values": { "key": value, ... } } */
    char *resp = malloc(2048);
    if (!resp) return strdup("{\"error\":\"OOM\"}");
    int off = 0;
    off += snprintf(resp + off, 2048 - off, "{\"values\":{");
    for (int i = 0; i < slice->entry_count; i++) {
        if (i > 0) off += snprintf(resp + off, 2048 - off, ",");
        off += snprintf(resp + off, 2048 - off, "\"%s\":%s",
                        slice->entries[i].key,
                        slice->entries[i].value_json[0]
                            ? slice->entries[i].value_json : "null");
    }
    off += snprintf(resp + off, 2048 - off, "}}");
    return resp;
}

/* ── __alloy_config_set bridge ──────────────────────────────────────────── */
/*
 * Called from JS: __alloy_config_set(pluginName, key, valueJson, persist)
 * Updates the in-memory config value for the plugin.
 * persist=true would write back to alloyconfig.toml (not yet implemented).
 */
static char *config_set_handler(const char *name, const char *args_json,
                                  void *userdata) {
    (void)name;
    ConfigPluginState *st = (ConfigPluginState *)userdata;

    /* args_json: [pluginName, key, valueJson, persist] */
    /* Minimal extraction */
    const char *p = args_json;
    if (*p == '[') p++;

    /* plugin_name */
    if (*p == '"') p++;
    char plugin_name[64] = {0};
    const char *end = strchr(p, '"');
    if (!end) return strdup("{\"error\":\"bad args\"}");
    size_t nlen = (size_t)(end - p);
    if (nlen >= sizeof(plugin_name)) nlen = sizeof(plugin_name) - 1;
    memcpy(plugin_name, p, nlen);
    p = end + 2; /* skip closing " and , */

    /* key */
    if (*p == '"') p++;
    char key[64] = {0};
    end = strchr(p, '"');
    if (!end) return strdup("{\"error\":\"bad args\"}");
    size_t klen = (size_t)(end - p);
    if (klen >= sizeof(key)) klen = sizeof(key) - 1;
    memcpy(key, p, klen);
    p = end + 2; /* skip closing " and , */

    /* value_json — everything up to the last , */
    char value_json[512] = {0};
    const char *val_start = p;
    const char *last_comma = strrchr(p, ',');
    size_t vlen = last_comma ? (size_t)(last_comma - val_start) : strlen(val_start);
    if (vlen >= sizeof(value_json)) vlen = sizeof(value_json) - 1;
    memcpy(value_json, val_start, vlen);

    PluginConfigSlice *slice = get_or_create_slice(st, plugin_name);
    if (!slice) return strdup("{\"error\":\"config registry full\"}");
    ConfigEntry *entry = get_or_create_entry(slice, key);
    if (!entry) return strdup("{\"error\":\"config key limit reached\"}");
    strncpy(entry->value_json, value_json, sizeof(entry->value_json) - 1);

    /* Sync back to Alloy.config in JS */
    if (st->engine) {
        char script[512];
        snprintf(script, sizeof(script),
            "if(Alloy&&Alloy.config&&Alloy.config.plugins&&Alloy.config.plugins[\"%s\"])"
            "  Alloy.config.plugins[\"%s\"][\"%s\"]=%s;",
            plugin_name, plugin_name, key, value_json);
        char *r = alloy_engine_eval(st->engine, script, "<config_set>");
        free(r);
    }

    return strdup("{\"success\":true}");
}

/* ── Plugin lifecycle ───────────────────────────────────────────────────── */

static int config_plugin_on_init(RuntimePlugin *plugin, AlloyJSEngine *eng) {
    ConfigPluginState *st = (ConfigPluginState *)plugin->userdata;
    st->engine = eng;

    /* Serialise runtime config → Alloy.config in JS */
    char *cfg_json = runtime_config_to_json(&st->runtime_cfg);
    if (!cfg_json) return ALLOY_ERR_OOM;

    char *script = malloc(strlen(cfg_json) + 128);
    if (!script) { free(cfg_json); return ALLOY_ERR_OOM; }
    sprintf(script, "Alloy.config = %s;", cfg_json);
    free(cfg_json);

    char *r = alloy_engine_eval(eng, script, "<config_init>");
    free(script);
    free(r);

    /* Register the two C bridges */
    int rc = alloy_engine_bind(eng, "__alloy_use_config", use_config_handler, st);
    if (rc != ALLOY_OK) return rc;
    return alloy_engine_bind(eng, "__alloy_config_set", config_set_handler, st);
}

static void config_plugin_on_destroy(RuntimePlugin *plugin, AlloyJSEngine *eng) {
    (void)eng;
    free(plugin->userdata);
}

RuntimePlugin *alloy_config_plugin_create(AlloyRuntimeConfig cfg) {
    ConfigPluginState *st = calloc(1, sizeof(ConfigPluginState));
    if (!st) return NULL;
    st->runtime_cfg = cfg;

    RuntimePlugin *p = calloc(1, sizeof(RuntimePlugin));
    if (!p) { free(st); return NULL; }
    p->name       = "alloy:config";
    p->version    = "0.1.0";
    p->on_init    = config_plugin_on_init;
    p->on_destroy = config_plugin_on_destroy;
    p->userdata   = st;
    return p;
}

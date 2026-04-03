#include "plugin.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>

typedef struct {
    char *name;
    char *version;
    char *entry;
    char *native;
    char **dependencies;
    int dep_count;
} PluginManifest;

static int is_valid_name(const char *name) {
    if (!name || !*name || !islower(*name)) return 0;
    for (const char *p = name; *p; p++) {
        if (!islower(*p) && !isdigit(*p) && *p != ':' && *p != '_' && *p != '-') return 0;
    }
    return 1;
}

int alloy_manifest_parse(const char *toml_content, PluginManifest *out) {
    // Basic TOML parser simulation for manifest
    // In production, we'd use smol-toml equivalent in C or a proper parser
    if (strstr(toml_content, "name = \"") == NULL) return ALLOY_ERR_MANIFEST;

    // Extraction logic...
    char name[128];
    sscanf(toml_content, "name = \"%[^\"]\"", name);
    if (!is_valid_name(name)) return ALLOY_ERR_MANIFEST;

    out->name = strdup(name);
    return ALLOY_OK;
}

char *alloy_manifest_to_toml(PluginManifest *m) {
    char *buf = malloc(1024);
    sprintf(buf, "name = \"%s\"\nversion = \"%s\"\nentry = \"%s\"\n", m->name, m->version, m->entry);
    return buf;
}

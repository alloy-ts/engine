/**
 * AlloyScript — Static Plugin Registry implementation
 *
 * A singly-linked list of AlloyPluginRegistryEntry nodes.  Nodes are
 * stack-allocated inside each plugin's constructor function and prepended
 * to g_registry_head at startup (before main()).  No heap allocation,
 * no locks — safe for use in constructors.
 *
 * engine_create.c calls alloy_plugin_registry_head() once during
 * alloy_engine_create_with_config() to walk the list and load every
 * registered factory in registration order (i.e. link order).
 */
#include "../../include/alloy/plugin_registry.h"
#include <stddef.h>

/* ── Global registry head ────────────────────────────────────────────────── */

static AlloyPluginRegistryEntry *g_registry_head = NULL;

/* ── API ─────────────────────────────────────────────────────────────────── */

void alloy_plugin_register(const char *name, AlloyPluginFactory factory) {
    if (!factory) return;

    /*
     * Nodes must outlive the registry walk.  Constructors run at static-init
     * time, so we allocate nodes in static storage inside the calling
     * translation unit (see ALLOY_PLUGIN_REGISTER macro — the node is
     * embedded in the constructor stack frame of a function that never
     * returns to a shorter-lived scope before main()).
     *
     * In practice each plugin calls this exactly once from a constructor,
     * so we use a small static pool here to keep things allocation-free.
     */
#define ALLOY_REGISTRY_MAX 32
    static AlloyPluginRegistryEntry node_pool[ALLOY_REGISTRY_MAX];
    static int pool_used = 0;

    if (pool_used >= ALLOY_REGISTRY_MAX) return; /* silently drop if full */

    AlloyPluginRegistryEntry *node = &node_pool[pool_used++];
    node->name    = name;
    node->factory = factory;
    node->next    = g_registry_head;
    g_registry_head = node;
}

AlloyPluginRegistryEntry *alloy_plugin_registry_head(void) {
    return g_registry_head;
}

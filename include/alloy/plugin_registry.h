/**
 * AlloyScript — Static Plugin Registry
 *
 * Provides a zero-cost self-registration mechanism for built-in plugins.
 * Every plugin that wants to be auto-loaded calls alloy_plugin_register()
 * from a constructor function (or a static initialiser on MSVC).
 * engine_create.c knows nothing about individual plugins — it only iterates
 * the registry list returned by alloy_plugin_registry_head().
 *
 * Usage in a plugin translation unit:
 *
 *   // at file scope, after the factory function:
 *   ALLOY_PLUGIN_REGISTER(my_plugin_factory, my_cap_store_ptr_holder)
 *
 * or, for plugins whose factory needs a cap_store at registration time
 * (cap_store is always NULL at constructor time — engine_create passes it
 * when it walks the registry):
 *
 *   static void __attribute__((constructor)) _my_plugin_auto_register(void) {
 *       alloy_plugin_register(my_plugin_factory, "my:plugin");
 *   }
 *
 * Thread safety: registration happens at process startup before main(),
 * so no locking is needed.
 */
#ifndef ALLOY_PLUGIN_REGISTRY_H
#define ALLOY_PLUGIN_REGISTRY_H

#include "plugin.h"
#include "capability.h"

#ifdef __cplusplus
extern "C" {
#endif

/* ── Factory function type ───────────────────────────────────────────────── */

/**
 * A plugin factory.  Called by engine_create once per runtime construction,
 * with the fully-initialised cap_store.  Returns a heap-allocated
 * RuntimePlugin* (freed by unload_plugins on teardown), or NULL to skip.
 */
typedef RuntimePlugin *(*AlloyPluginFactory)(AlloyCapStore *cap_store);

/* ── Registry node ───────────────────────────────────────────────────────── */

typedef struct AlloyPluginRegistryEntry {
    const char                       *name;     /* e.g. "alloy:web"  */
    AlloyPluginFactory                factory;  /* creates the plugin */
    struct AlloyPluginRegistryEntry  *next;
} AlloyPluginRegistryEntry;

/* ── API ─────────────────────────────────────────────────────────────────── */

/**
 * Register a plugin factory.  Call this from a constructor function or
 * static initialiser — never from application code.
 *
 * @param name     Human-readable plugin name (stored by pointer, must be
 *                 a string literal or have static lifetime).
 * @param factory  Function that constructs the RuntimePlugin*.
 */
void alloy_plugin_register(const char *name, AlloyPluginFactory factory);

/**
 * Returns the head of the singly-linked registry list.
 * Called once by engine_create during runtime initialisation.
 */
AlloyPluginRegistryEntry *alloy_plugin_registry_head(void);

/* ── Convenience macro ───────────────────────────────────────────────────── */

/**
 * ALLOY_PLUGIN_REGISTER(factory_fn, plugin_name_literal)
 *
 * Declares a constructor that registers `factory_fn` at startup.
 * Place this at file scope in the plugin's .c / .m file.
 *
 * On GCC/Clang: uses __attribute__((constructor)).
 * On MSVC:      uses #pragma section + pointer in .CRT$XCU.
 */
#if defined(_MSC_VER)
#  define ALLOY_PLUGIN_REGISTER(factory_fn, plugin_name)                  \
     static void _alloy_reg_##factory_fn(void);                           \
     __pragma(section(".CRT$XCU", read))                                  \
     __declspec(allocate(".CRT$XCU"))                                      \
     static void (*_alloy_reg_ptr_##factory_fn)(void) =                   \
         _alloy_reg_##factory_fn;                                          \
     static void _alloy_reg_##factory_fn(void) {                          \
         alloy_plugin_register((plugin_name), (factory_fn));               \
     }
#else
#  define ALLOY_PLUGIN_REGISTER(factory_fn, plugin_name)                  \
     __attribute__((constructor))                                          \
     static void _alloy_reg_##factory_fn(void) {                          \
         alloy_plugin_register((plugin_name), (factory_fn));               \
     }
#endif

#ifdef __cplusplus
}
#endif

#endif /* ALLOY_PLUGIN_REGISTRY_H */

/**
 * AlloyScript — Capability Model
 *
 * Capabilities are signed tokens that gate every IPC request from the
 * hidden web process (and future renderers) to the trusted core.
 * The central capability store lives ONLY in the main core process.
 */
#ifndef ALLOY_CAPABILITY_H
#define ALLOY_CAPABILITY_H

#include <stdint.h>
#include <stdbool.h>
#include <time.h>

#ifdef __cplusplus
extern "C" {
#endif

#define ALLOY_CAP_ID_LEN      32   /* hex string */
#define ALLOY_CAP_RESOURCE_LEN 64
#define ALLOY_CAP_SCOPE_LEN   256
#define ALLOY_CAP_ISSUED_TO_LEN 64
#define ALLOY_CAP_MAX_SCOPES   8

/* Permission bitmask */
typedef enum {
    ALLOY_PERM_NONE    = 0,
    ALLOY_PERM_READ    = (1 << 0),
    ALLOY_PERM_WRITE   = (1 << 1),
    ALLOY_PERM_EXECUTE = (1 << 2),
    ALLOY_PERM_NETWORK = (1 << 3),
} AlloyPermission;

/**
 * A capability token. Issued by the core; validated on every IPC call.
 * In production these would be HMAC-signed; here we use a simple
 * in-memory store with constant-time lookup.
 */
typedef struct {
    char     cap_id[ALLOY_CAP_ID_LEN + 1];
    char     resource[ALLOY_CAP_RESOURCE_LEN];   /* e.g. "web:fetch" */
    uint32_t permissions;                         /* AlloyPermission bitmask */
    char     scopes[ALLOY_CAP_MAX_SCOPES][ALLOY_CAP_SCOPE_LEN]; /* URL/path patterns */
    int      scope_count;
    char     issued_to[ALLOY_CAP_ISSUED_TO_LEN]; /* process/component name */
    time_t   expires_at;                          /* 0 = no expiry */
    bool     revoked;
} AlloyCap;

/* Opaque capability store — lives in the core process only */
typedef struct AlloyCapStore AlloyCapStore;

AlloyCapStore *alloy_cap_store_create(void);
void           alloy_cap_store_destroy(AlloyCapStore *store);

/**
 * Issue a new capability. Returns a heap-allocated cap_id string (caller frees).
 * Returns NULL on failure.
 */
char *alloy_cap_issue(AlloyCapStore *store, const char *resource,
                      uint32_t permissions, const char **scopes, int scope_count,
                      const char *issued_to, time_t expires_at);

/**
 * Validate a capability for a given resource + permission.
 * Returns true if valid, false if denied (expired, revoked, wrong resource, etc.).
 */
bool alloy_cap_validate(AlloyCapStore *store, const char *cap_id,
                        const char *resource, uint32_t required_perm,
                        const char *target_scope);

/** Revoke a capability by ID. */
void alloy_cap_revoke(AlloyCapStore *store, const char *cap_id);

#ifdef __cplusplus
}
#endif

#endif /* ALLOY_CAPABILITY_H */

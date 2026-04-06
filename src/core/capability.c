/**
 * AlloyScript — Capability Store
 *
 * Central in-memory capability store. Lives ONLY in the main core process.
 * All IPC requests from the hidden web process must include a valid cap_id.
 *
 * Security properties:
 * - Constant-time cap_id comparison (prevents timing attacks).
 * - Scope matching supports exact match and wildcard prefix (e.g. "https://api.example.com/*").
 * - Expiry checked against wall clock.
 */
#include "../../include/alloy/capability.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <time.h>

#ifdef __APPLE__
#  include <CommonCrypto/CommonRandom.h>
#  define ALLOY_RAND_BYTES(buf, len) CCRandomGenerateBytes(buf, len)
#else
#  include <sys/random.h>
#  define ALLOY_RAND_BYTES(buf, len) getrandom(buf, len, 0)
#endif

#define CAP_STORE_MAX 256

struct AlloyCapStore {
    AlloyCap caps[CAP_STORE_MAX];
    int      count;
};

/* ── Helpers ────────────────────────────────────────────────────────────── */

/* Constant-time string comparison (same length required) */
static int ct_strcmp(const char *a, const char *b, size_t len) {
    int diff = 0;
    for (size_t i = 0; i < len; i++) diff |= (unsigned char)a[i] ^ (unsigned char)b[i];
    return diff;
}

/* Generate a random hex cap_id (32 hex chars = 16 bytes entropy) */
static void gen_cap_id(char out[ALLOY_CAP_ID_LEN + 1]) {
    uint8_t raw[16];
    ALLOY_RAND_BYTES(raw, sizeof(raw));
    static const char hex[] = "0123456789abcdef";
    for (int i = 0; i < 16; i++) {
        out[i * 2]     = hex[raw[i] >> 4];
        out[i * 2 + 1] = hex[raw[i] & 0xf];
    }
    out[32] = '\0';
}

/* Check if target matches a scope pattern (exact or "prefix/*") */
static bool scope_matches(const char *pattern, const char *target) {
    if (!pattern || !target) return false;
    size_t plen = strlen(pattern);
    if (plen > 2 && pattern[plen - 2] == '/' && pattern[plen - 1] == '*') {
        /* Prefix wildcard: pattern = "https://api.example.com/*" */
        return strncmp(pattern, target, plen - 1) == 0;
    }
    return strcmp(pattern, target) == 0;
}

/* ── Public API ─────────────────────────────────────────────────────────── */

AlloyCapStore *alloy_cap_store_create(void) {
    AlloyCapStore *s = calloc(1, sizeof(AlloyCapStore));
    return s;
}

void alloy_cap_store_destroy(AlloyCapStore *store) {
    if (store) free(store);
}

char *alloy_cap_issue(AlloyCapStore *store, const char *resource,
                      uint32_t permissions, const char **scopes, int scope_count,
                      const char *issued_to, time_t expires_at) {
    if (!store || !resource || store->count >= CAP_STORE_MAX) return NULL;

    AlloyCap *cap = &store->caps[store->count++];
    memset(cap, 0, sizeof(*cap));

    gen_cap_id(cap->cap_id);
    strncpy(cap->resource, resource, ALLOY_CAP_RESOURCE_LEN - 1);
    cap->permissions = permissions;
    cap->expires_at  = expires_at;
    cap->revoked     = false;

    if (issued_to)
        strncpy(cap->issued_to, issued_to, ALLOY_CAP_ISSUED_TO_LEN - 1);

    int n = scope_count < ALLOY_CAP_MAX_SCOPES ? scope_count : ALLOY_CAP_MAX_SCOPES;
    for (int i = 0; i < n; i++) {
        if (scopes[i])
            strncpy(cap->scopes[i], scopes[i], ALLOY_CAP_SCOPE_LEN - 1);
    }
    cap->scope_count = n;

    return strdup(cap->cap_id);
}

bool alloy_cap_validate(AlloyCapStore *store, const char *cap_id,
                        const char *resource, uint32_t required_perm,
                        const char *target_scope) {
    if (!store || !cap_id || !resource) return false;

    size_t id_len = strlen(cap_id);
    if (id_len != ALLOY_CAP_ID_LEN) return false;

    time_t now = time(NULL);

    for (int i = 0; i < store->count; i++) {
        AlloyCap *cap = &store->caps[i];

        /* Constant-time ID comparison */
        if (ct_strcmp(cap->cap_id, cap_id, ALLOY_CAP_ID_LEN) != 0) continue;

        if (cap->revoked) return false;
        if (cap->expires_at != 0 && now > cap->expires_at) return false;
        if (strcmp(cap->resource, resource) != 0) return false;
        if ((cap->permissions & required_perm) != required_perm) return false;

        /* Scope check: if target_scope provided, at least one scope must match */
        if (target_scope && cap->scope_count > 0) {
            bool scope_ok = false;
            for (int j = 0; j < cap->scope_count; j++) {
                if (scope_matches(cap->scopes[j], target_scope)) {
                    scope_ok = true;
                    break;
                }
            }
            if (!scope_ok) return false;
        }

        return true;
    }
    return false;
}

void alloy_cap_revoke(AlloyCapStore *store, const char *cap_id) {
    if (!store || !cap_id) return;
    size_t id_len = strlen(cap_id);
    if (id_len != ALLOY_CAP_ID_LEN) return;

    for (int i = 0; i < store->count; i++) {
        if (ct_strcmp(store->caps[i].cap_id, cap_id, ALLOY_CAP_ID_LEN) == 0) {
            store->caps[i].revoked = true;
            return;
        }
    }
}

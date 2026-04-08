#include "alloy/capability.h"
#include <stdlib.h>
#include <string.h>

/* Task 7.1: Create src/core/capability/store.c */

typedef struct {
    AlloyCapabilityToken *tokens;
    size_t count;
    uint8_t issuer_public_key[32];
} AlloyCapabilityStore;

int alloy_capability_store_init(AlloyCapabilityStore *store, const uint8_t *issuer_key) {
    store->tokens = NULL;
    store->count = 0;
    memcpy(store->issuer_public_key, issuer_key, 32);
    return 0;
}

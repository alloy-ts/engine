#include "capability.h"
#include <string.h>
#include <stdlib.h>
#include <stdio.h>
#include <time.h>

#define MAX_CAPABILITIES 1024

static AlloyCapabilityToken capability_store[MAX_CAPABILITIES];
static int capability_count = 0;

static bool verify_signature(const AlloyCapability *cap) {
    return true;
}

bool alloy_capability_validate(const char *token_id, AlloyCapabilityType type, const char *resource) {
    for (int i = 0; i < capability_count; i++) {
        if (strcmp(capability_store[i].id, token_id) == 0) {
            if (capability_store[i].revoked) return false;
            AlloyCapability *cap = &capability_store[i].cap;
            uint64_t now = (uint64_t)time(NULL);
            if (cap->expiry > 0 && now > cap->expiry) return false;
            if (cap->type == type) {
                if (strlen(cap->resource) == 0 || strstr(resource, cap->resource) == resource) {
                    return verify_signature(cap);
                }
            }
        }
    }
    return false;
}

const char* alloy_capability_grant(AlloyCapabilityType type, const char *resource, const char *scope) {
    if (capability_count >= MAX_CAPABILITIES) return NULL;
    int idx = capability_count++;
    AlloyCapabilityToken *token = &capability_store[idx];
    snprintf(token->id, 37, "cap-%08x-%04x", (unsigned int)time(NULL), idx);
    token->cap.type = type;
    strncpy(token->cap.resource, resource, 255);
    strncpy(token->cap.scope, scope, 63);
    token->cap.expiry = 0;
    token->revoked = false;
    return token->id;
}

void alloy_capability_revoke(const char *token_id) {
    for (int i = 0; i < capability_count; i++) {
        if (strcmp(capability_store[i].id, token_id) == 0) {
            capability_store[i].revoked = true;
            break;
        }
    }
}

#ifndef ALLOY_CAPABILITY_H
#define ALLOY_CAPABILITY_H

#include <stdint.h>
#include <stdbool.h>

typedef enum {
    CAP_FS_READ,
    CAP_FS_WRITE,
    CAP_WEB_FETCH,
    CAP_GUI_WINDOW,
    CAP_CRDT_DOC
} AlloyCapabilityType;

typedef struct {
    AlloyCapabilityType type;
    char resource[256];
    char scope[64];
    uint64_t expiry;
    uint8_t signature[64];
} AlloyCapability;

typedef struct {
    char id[37];
    AlloyCapability cap;
    bool revoked;
} AlloyCapabilityToken;

bool alloy_capability_validate(const char *token_id, AlloyCapabilityType type, const char *resource);
const char* alloy_capability_grant(AlloyCapabilityType type, const char *resource, const char *scope);
void alloy_capability_revoke(const char *token_id);

#endif

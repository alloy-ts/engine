#ifndef ALLOY_CAPABILITY_H
#define ALLOY_CAPABILITY_H

#include <stdint.h>

#define ALLOY_CAP_SIZE 256

typedef enum {
    CAP_NONE  = 0,
    CAP_READ  = 1 << 0,
    CAP_WRITE = 1 << 1,
    CAP_EXEC  = 1 << 2,
} AlloyPermission;

typedef struct {
    uint8_t cap_id[32];
    uint8_t resource[64];
    uint32_t permissions;
    uint8_t scopes[512];
    uint64_t issued_to;
    uint64_t issued_at;
    uint64_t expires_at;
    uint8_t issuer_key[32];
    uint8_t signature[64];
} AlloyCapabilityToken;

#endif

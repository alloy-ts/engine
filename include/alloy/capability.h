#ifndef ALLOY_CAPABILITY_H
#define ALLOY_CAPABILITY_H
#include <stdint.h>
#include <stdbool.h>
typedef struct { uint8_t cap_id[32]; uint8_t resource[64]; uint32_t permissions; uint8_t scopes[512]; uint8_t signature[64]; } AlloyCapabilityToken;
bool alloy_capability_validate(const AlloyCapabilityToken *token, const char *resource, uint32_t perms);
#endif

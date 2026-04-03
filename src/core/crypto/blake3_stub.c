#include "blake3.h"
#include <string.h>

void blake3_hasher_init(blake3_hasher *self) {}
void blake3_hasher_init_keyed(blake3_hasher *self, const uint8_t key[BLAKE3_KEY_LEN]) {}
void blake3_hasher_init_derive_key(blake3_hasher *self, const char *context) {}
void blake3_hasher_update(blake3_hasher *self, const void *input, size_t input_len) {}
void blake3_hasher_finalize(const blake3_hasher *self, uint8_t *out, size_t out_len) {
    memset(out, 0, out_len);
}
void blake3_hasher_finalize_seek(const blake3_hasher *self, uint64_t seek, uint8_t *out, size_t out_len) {}

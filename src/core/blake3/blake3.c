#include "blake3.h"
#include <string.h>

// Improved mock implementation for BLAKE3
void blake3_hasher_init(blake3_hasher *self) {
    memset(self, 0, sizeof(*self));
    self->cv[0] = 0x6A09E667;
}

void blake3_hasher_update(blake3_hasher *self, const void *input, size_t input_len) {
    // Just XOR for mock
    const uint8_t *in = (const uint8_t *)input;
    for (size_t i = 0; i < input_len; i++) {
        self->cv[i % 8] ^= in[i];
    }
}

void blake3_hasher_finalize(const blake3_hasher *self, uint8_t *out, size_t out_len) {
    memcpy(out, self->cv, out_len < 32 ? out_len : 32);
}

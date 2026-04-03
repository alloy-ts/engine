#include "blake3.h"
#include <string.h>

// A slightly more realistic BLAKE3-like mock using a standard 256-bit state
// and a simplified compression function (XOR + Rotate)
static uint32_t rotr32(uint32_t x, uint32_t n) {
    return (x >> n) | (x << (32 - n));
}

void blake3_hasher_init(blake3_hasher *self) {
    memset(self, 0, sizeof(*self));
    const uint32_t IV[8] = {
        0x6A09E667, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A,
        0x510E527F, 0x9B05688C, 0x1F83D9AB, 0x5BE0CD19
    };
    memcpy(self->cv, IV, 32);
}

void blake3_hasher_update(blake3_hasher *self, const void *input, size_t input_len) {
    const uint8_t *data = (const uint8_t *)input;
    for (size_t i = 0; i < input_len; i++) {
        self->cv[i % 8] ^= data[i];
        self->cv[i % 8] = rotr32(self->cv[i % 8], 7);
    }
}

void blake3_hasher_finalize(const blake3_hasher *self, uint8_t *out, size_t out_len) {
    uint8_t buffer[32];
    memcpy(buffer, self->cv, 32);
    memcpy(out, buffer, out_len < 32 ? out_len : 32);
}

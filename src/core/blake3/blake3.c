#include "blake3_impl.h"

INLINE uint32_t rotr32(uint32_t w, uint32_t c) {
  return (w >> c) | (w << (32 - c));
}

INLINE void g(uint32_t *state, size_t a, size_t b, size_t c, size_t d,
              uint32_t x, uint32_t y) {
  state[a] = state[a] + state[b] + x;
  state[d] = rotr32(state[d] ^ state[a], 16);
  state[c] = state[c] + state[d];
  state[b] = rotr32(state[b] ^ state[c], 12);
  state[a] = state[a] + state[b] + y;
  state[d] = rotr32(state[d] ^ state[a], 8);
  state[c] = state[c] + state[d];
  state[b] = rotr32(state[b] ^ state[c], 7);
}

INLINE void round_fn(uint32_t state[16], const uint32_t *msg, size_t round) {
  const uint8_t *schedule = MSG_SCHEDULE[round];
  g(state, 0, 4, 8, 12, msg[schedule[0]], msg[schedule[1]]);
  g(state, 1, 5, 9, 13, msg[schedule[2]], msg[schedule[3]]);
  g(state, 2, 6, 10, 14, msg[schedule[4]], msg[schedule[5]]);
  g(state, 3, 7, 11, 15, msg[schedule[6]], msg[schedule[7]]);
  g(state, 0, 5, 10, 15, msg[schedule[8]], msg[schedule[9]]);
  g(state, 1, 6, 11, 12, msg[schedule[10]], msg[schedule[11]]);
  g(state, 2, 7, 8, 13, msg[schedule[12]], msg[schedule[13]]);
  g(state, 3, 4, 9, 14, msg[schedule[14]], msg[schedule[15]]);
}

INLINE void compress_pre(uint32_t state[16], const uint32_t cv[8],
                         const uint8_t block[BLAKE3_BLOCK_LEN],
                         uint8_t block_len, uint64_t counter, uint8_t flags) {
  uint32_t block_words[16];
  for (int i = 0; i < 16; i++) block_words[i] = load32(block + 4 * i);
  state[0] = cv[0]; state[1] = cv[1]; state[2] = cv[2]; state[3] = cv[3];
  state[4] = cv[4]; state[5] = cv[5]; state[6] = cv[6]; state[7] = cv[7];
  state[8] = IV[0]; state[9] = IV[1]; state[10] = IV[2]; state[11] = IV[3];
  state[12] = counter_low(counter); state[13] = counter_high(counter);
  state[14] = (uint32_t)block_len; state[15] = (uint32_t)flags;
  for (int i = 0; i < 7; i++) round_fn(state, block_words, i);
}

void blake3_compress_in_place_portable(uint32_t cv[8],
                                       const uint8_t block[BLAKE3_BLOCK_LEN],
                                       uint8_t block_len, uint64_t counter,
                                       uint8_t flags) {
  uint32_t state[16];
  compress_pre(state, cv, block, block_len, counter, flags);
  for (int i = 0; i < 8; i++) cv[i] = state[i] ^ state[i + 8];
}

void blake3_compress_xof_portable(const uint32_t cv[8],
                                  const uint8_t block[BLAKE3_BLOCK_LEN],
                                  uint8_t block_len, uint64_t counter,
                                  uint8_t flags, uint8_t out[64]) {
  uint32_t state[16];
  compress_pre(state, cv, block, block_len, counter, flags);
  for (int i = 0; i < 8; i++) store32(&out[i * 4], state[i] ^ state[i + 8]);
  for (int i = 0; i < 8; i++) store32(&out[(i + 8) * 4], state[i + 8] ^ cv[i]);
}

void blake3_hasher_init(blake3_hasher *self) {
    memcpy(self->cv, IV, 32);
    self->chunk_counter = 0;
    self->buf_len = 0;
    self->blocks_compressed = 0;
    self->flags = 0;
}

void blake3_hasher_update(blake3_hasher *self, const void *input, size_t input_len) {
    const uint8_t *p = (const uint8_t *)input;
    while (input_len > 0) {
        if (self->buf_len == BLAKE3_BLOCK_LEN) {
            blake3_compress_in_place_portable(self->cv, self->buf, BLAKE3_BLOCK_LEN, self->chunk_counter, self->flags);
            self->buf_len = 0;
            self->blocks_compressed++;
        }
        size_t take = BLAKE3_BLOCK_LEN - self->buf_len;
        if (take > input_len) take = input_len;
        memcpy(self->buf + self->buf_len, p, take);
        p += take; self->buf_len += (uint8_t)take; input_len -= take;
    }
}

void blake3_hasher_finalize(const blake3_hasher *self, uint8_t *out, size_t out_len) {
    uint32_t cv[8]; memcpy(cv, self->cv, 32);
    uint8_t flags = self->flags | 0x08; // ROOT flag simplified
    blake3_compress_in_place_portable(cv, self->buf, self->buf_len, self->chunk_counter, flags);
    for (int i = 0; i < 8; i++) store32(out + i * 4, cv[i]);
}

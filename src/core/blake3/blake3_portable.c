#include "blake3_impl.h"
#include <string.h>

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

void blake3_hash_many_portable(const uint8_t *const *inputs, size_t num_inputs,
                               size_t blocks, const uint32_t key[8],
                               uint64_t counter, bool increment_counter,
                               uint8_t flags, uint8_t flags_start,
                               uint8_t flags_end, uint8_t *out) {
  while (num_inputs > 0) {
    uint32_t cv[8]; memcpy(cv, key, 32);
    uint8_t block_flags = flags | flags_start;
    const uint8_t *input = inputs[0];
    for (size_t i = 0; i < blocks; i++) {
        if (i == blocks - 1) block_flags |= flags_end;
        blake3_compress_in_place_portable(cv, input, 64, counter, block_flags);
        input += 64; block_flags = flags;
    }
    store_cv_words(out, cv);
    if (increment_counter) counter++;
    inputs++; num_inputs--; out += 32;
  }
}
void blake3_compress_in_place(uint32_t cv[8], const uint8_t block[64], uint8_t block_len, uint64_t counter, uint8_t flags) {
    blake3_compress_in_place_portable(cv, block, block_len, counter, flags);
}
void blake3_compress_xof(const uint32_t cv[8], const uint8_t block[64], uint8_t block_len, uint64_t counter, uint8_t flags, uint8_t out[64]) {
    blake3_compress_xof_portable(cv, block, block_len, counter, flags, out);
}
void blake3_xof_many(const uint32_t cv[8], const uint8_t block[64], uint8_t block_len, uint64_t counter, uint8_t flags, uint8_t out[64], size_t outblocks) {
    while (outblocks--) {
        blake3_compress_xof(cv, block, block_len, counter++, flags, out);
        out += 64;
    }
}
void blake3_hash_many(const uint8_t *const *inputs, size_t num_inputs, size_t blocks, const uint32_t key[8], uint64_t counter, bool increment_counter, uint8_t flags, uint8_t flags_start, uint8_t flags_end, uint8_t *out) {
    blake3_hash_many_portable(inputs, num_inputs, blocks, key, counter, increment_counter, flags, flags_start, flags_end, out);
}
size_t blake3_simd_degree(void) { return 1; }

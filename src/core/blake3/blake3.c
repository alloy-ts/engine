#include "blake3.h"
#include "blake3_impl.h"

const char *blake3_version(void) { return BLAKE3_VERSION_STRING; }

INLINE void chunk_state_init(blake3_chunk_state *self, const uint32_t key[8], uint8_t flags) {
  memcpy(self->cv, key, 32);
  self->chunk_counter = 0;
  memset(self->buf, 0, 64);
  self->buf_len = 0;
  self->blocks_compressed = 0;
  self->flags = flags;
}

INLINE void chunk_state_update(blake3_chunk_state *self, const uint8_t *input, size_t input_len) {
  while (input_len > 0) {
    if (self->buf_len == 64) {
      blake3_compress_in_place(self->cv, self->buf, 64, self->chunk_counter, self->flags | (self->blocks_compressed == 0 ? CHUNK_START : 0));
      self->blocks_compressed++;
      self->buf_len = 0;
    }
    size_t want = 64 - self->buf_len;
    size_t take = input_len < want ? input_len : want;
    memcpy(self->buf + self->buf_len, input, take);
    self->buf_len += (uint8_t)take;
    input += take;
    input_len -= take;
  }
}

void blake3_hasher_init(blake3_hasher *self) {
  load_key_words((const uint8_t *)IV, self->key);
  chunk_state_init(&self->chunk, self->key, 0);
  self->cv_stack_len = 0;
}

void blake3_hasher_update(blake3_hasher *self, const void *input, size_t input_len) {
  chunk_state_update(&self->chunk, (const uint8_t *)input, input_len);
}

void blake3_hasher_finalize(const blake3_hasher *self, uint8_t *out, size_t out_len) {
  uint32_t cv[8]; memcpy(cv, self->chunk.cv, 32);
  blake3_compress_in_place(cv, self->chunk.buf, self->chunk.buf_len, self->chunk.chunk_counter, self->chunk.flags | (self->chunk.blocks_compressed == 0 ? CHUNK_START : 0) | CHUNK_END | ROOT);
  store_cv_words(out, cv);
}

#ifndef BLAKE3_H
#define BLAKE3_H

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

#define BLAKE3_KEY_LEN 32
#define BLAKE3_OUT_LEN 32
#define BLAKE3_BLOCK_LEN 64
#define BLAKE3_CHUNK_LEN 1024
#define BLAKE3_MAX_DEPTH 54

typedef struct {
  uint32_t cv[8];
  uint64_t chunk_counter;
  uint8_t buf[BLAKE3_BLOCK_LEN];
  uint8_t buf_len;
  uint8_t blocks_compressed;
  uint8_t flags;
} blake3_chunk_state;

typedef struct {
  uint32_t key[8];
  blake3_chunk_state chunk;
  uint32_t cv_stack[BLAKE3_MAX_DEPTH][8];
  uint8_t cv_stack_len;
  uint8_t flags;
} blake3_hasher;

void blake3_hasher_init(blake3_hasher *self);
void blake3_hasher_init_keyed(blake3_hasher *self, const uint8_t key[BLAKE3_KEY_LEN]);
void blake3_hasher_init_derive_key(blake3_hasher *self, const char *context);
void blake3_hasher_update(blake3_hasher *self, const void *input, size_t input_len);
void blake3_hasher_finalize(const blake3_hasher *self, uint8_t *out, size_t out_len);
void blake3_hasher_finalize_seek(const blake3_hasher *self, uint64_t seek, uint8_t *out, size_t out_len);

#endif

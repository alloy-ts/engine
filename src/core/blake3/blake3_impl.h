#ifndef BLAKE3_IMPL_H
#define BLAKE3_IMPL_H

#include "blake3.h"
#include <string.h>

#if defined(_MSC_VER)
#define INLINE __forceinline
#else
#define INLINE static inline
#endif

static const uint32_t IV[8] = {
    0x6A09E667UL, 0xBB67AE85UL, 0x3C6EF372UL, 0xA54FF53AUL,
    0x510E527FUL, 0x9B05688CUL, 0x1F83D9ABUL, 0x5BE0CD19UL,
};

static const uint8_t MSG_SCHEDULE[7][16] = {
    {0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15},
    {2, 6, 3, 10, 7, 0, 4, 13, 1, 11, 12, 5, 9, 14, 15, 8},
    {3, 4, 10, 12, 13, 2, 7, 14, 6, 5, 9, 0, 11, 15, 8, 1},
    {10, 7, 12, 9, 14, 3, 13, 15, 4, 0, 11, 2, 5, 8, 1, 6},
    {12, 13, 9, 11, 15, 10, 14, 8, 7, 2, 5, 3, 0, 1, 6, 4},
    {9, 14, 11, 5, 8, 12, 15, 1, 13, 3, 0, 10, 2, 6, 4, 7},
    {11, 15, 5, 0, 1, 9, 8, 2, 14, 10, 2, 12, 3, 4, 7, 13},
};

INLINE uint32_t load32(const void *src) {
  uint32_t w;
  memcpy(&w, src, sizeof(w));
  return w;
}

INLINE void store32(void *dst, uint32_t w) {
  memcpy(dst, &w, sizeof(w));
}

INLINE uint32_t counter_low(uint64_t counter) { return (uint32_t)counter; }
INLINE uint32_t counter_high(uint64_t counter) { return (uint32_t)(counter >> 32); }

#endif

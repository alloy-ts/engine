#include <sodium.h>
#include <stdint.h>

/* Task 8.1: Create src/core/crypto/crypto.c */

int alloy_crypto_init(void) {
    if (sodium_init() < 0) return -1;
    return 0;
}

void alloy_crypto_box_keypair(uint8_t *pk, uint8_t *sk) {
    crypto_box_keypair(pk, sk);
}

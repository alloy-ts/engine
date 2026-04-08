#include <sodium.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>

/* Task 6: Encrypted RAM Manager */

typedef struct {
    uint8_t master_key[crypto_secretbox_KEYBYTES];
} AlloyEncryptedRamState;

static AlloyEncryptedRamState g_ram_state;

int alloy_encrypted_ram_init(void) {
    if (sodium_init() < 0) return -1;
    crypto_generichash(g_ram_state.master_key, sizeof(g_ram_state.master_key), (const uint8_t*)"alloy:master", 12, NULL, 0);
    return 0;
}

void *alloy_encrypted_ram_alloc(size_t size) {
    void *ptr = mmap(NULL, size, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    if (ptr == MAP_FAILED) return NULL;
    /* In a real implementation, we'd use XTS mode to encrypt/decrypt on access or at rest */
    return ptr;
}

void alloy_encrypted_ram_free(void *ptr, size_t size) {
    if (ptr) {
        sodium_memzero(ptr, size);
        munmap(ptr, size);
    }
}

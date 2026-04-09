#include <sodium.h>
#include <stdlib.h>
void* alloy_encrypted_ram_alloc(size_t size) { if (sodium_init() < 0) return NULL; return sodium_malloc(size); }
void alloy_encrypted_ram_free(void *ptr) { if (ptr) sodium_free(ptr); }

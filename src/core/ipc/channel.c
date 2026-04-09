#include <sodium.h>
#include <string.h>
typedef struct { uint8_t tx[32]; uint8_t rx[32]; uint64_t seq; } IPCChannel;
int alloy_ipc_encrypt(IPCChannel *c, uint8_t *p, size_t l, uint8_t *out) { uint8_t n[24]={0}; memcpy(n, &c->seq, 8); c->seq++; return crypto_aead_xchacha20poly1305_ietf_encrypt(out, NULL, p, l, NULL, 0, NULL, n, c->tx); }

#include "../../include/alloy/runtime.h"
#include "../../packages/mquickjs/mquickjs.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <sodium.h>

#define ALLOY_MAGIC 0x594C4C41

typedef struct {
    uint32_t magic;
    uint8_t  version;
    uint8_t  pubkey[32];
    uint8_t  sig[64];
    uint32_t payload_len;
} AlloyBlobHeader;

/**
 * alloy_load_blob
 * Verifies signature and loads bytecode into the engine.
 */
int alloy_load_blob(AlloyRuntime *rt, const uint8_t *data, size_t len) {
    if (len < sizeof(AlloyBlobHeader)) return -1;

    AlloyBlobHeader *header = (AlloyBlobHeader *)data;
    if (header->magic != ALLOY_MAGIC) return -1;

    // Verify signature
    // Sign region: magic + version + pubkey + payload_len + payload
    crypto_sign_state state;
    crypto_sign_init(&state);
    crypto_sign_update(&state, (uint8_t*)&header->magic, 4);
    crypto_sign_update(&state, &header->version, 1);
    crypto_sign_update(&state, header->pubkey, 32);
    crypto_sign_update(&state, (uint8_t*)&header->payload_len, 4);
    crypto_sign_update(&state, data + sizeof(AlloyBlobHeader), header->payload_len);

    if (crypto_sign_final_verify(&state, header->sig, header->pubkey) != 0) {
        fprintf(stderr, "[alloy:blob] Signature verification failed\n");
        return -1;
    }

    uint8_t *payload = (uint8_t *)(data + sizeof(AlloyBlobHeader));
    uint32_t payload_len = header->payload_len;

    if (payload_len + sizeof(AlloyBlobHeader) > len) return -1;

    // Load MQJS Bytecode
    JSContext *ctx = (JSContext *)rt->engine->engine;
    if (JS_RelocateBytecode(ctx, payload, payload_len) != 0) {
        return -1;
    }
    JSValue val = JS_LoadBytecode(ctx, payload);
    if (JS_IsException(val)) {
        return -1;
    }
    JSValue res = JS_Run(ctx, val);
    if (JS_IsException(res)) {
        return -1;
    }

    return 0;
}

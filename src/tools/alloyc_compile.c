#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include "../../packages/mquickjs/mquickjs.h"
#include <sodium.h>

/**
 * alloyc_compile
 * Compiles JS to MQJ bytecode, signs with Ed25519, and writes an Alloy_Blob.
 */

#define ALLOY_MAGIC 0x594C4C41 // "ALLY" LE

typedef struct {
    uint32_t magic;
    uint8_t  version;
    uint8_t  pubkey[32];
    uint8_t  sig[64];
    uint32_t payload_len;
} AlloyBlobHeader;

int main(int argc, char **argv) {
    if (argc < 5) {
        fprintf(stderr, "Usage: alloyc_compile -i <input.js> -o <output.alloy> --key <signing.key>\n");
        return 1;
    }

    const char *input_path = NULL;
    const char *output_path = NULL;
    const char *key_path = NULL;

    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "-i") == 0) input_path = argv[++i];
        else if (strcmp(argv[i], "-o") == 0) output_path = argv[++i];
        else if (strcmp(argv[i], "--key") == 0) key_path = argv[++i];
    }

    if (!input_path || !output_path || !key_path) return 1;

    if (sodium_init() < 0) return 1;

    // Load JS
    FILE *fin = fopen(input_path, "r");
    if (!fin) return 1;
    fseek(fin, 0, SEEK_END);
    long js_len = ftell(fin);
    fseek(fin, 0, SEEK_SET);
    char *js_src = malloc(js_len + 1);
    fread(js_src, 1, js_len, fin);
    js_src[js_len] = '\0';
    fclose(fin);

    // MQJS Bytecode compilation
    JSContext *ctx = JS_NewContext2(NULL, 0, NULL, JS_TRUE);
    if (!ctx) return 1;

    JSValue eval_code = JS_Eval(ctx, js_src, js_len, input_path, 0);
    if (JS_IsException(eval_code)) {
        JSValue exc = JS_GetException(ctx);
        JSCStringBuf buf;
        fprintf(stderr, "MQJS Compile Error: %s\n", JS_ToCString(ctx, exc, &buf));
        return 1;
    }

    JSBytecodeHeader b_hdr;
    const uint8_t *b_payload;
    uint32_t b_len;
    JS_PrepareBytecode(ctx, &b_hdr, &b_payload, &b_len, eval_code);

    // Header
    AlloyBlobHeader header = {0};
    header.magic = ALLOY_MAGIC;
    header.version = 0x01; // CPS/unencrypted
    header.payload_len = b_len;

    // Sign
    uint8_t privkey[crypto_sign_SECRETKEYBYTES];
    uint8_t pubkey[crypto_sign_PUBLICKEYBYTES];

    // For the prototype, if key file doesn't exist, we generate a dummy one
    FILE *kfile = fopen(key_path, "rb");
    if (!kfile) {
        crypto_sign_keypair(pubkey, privkey);
        kfile = fopen(key_path, "wb");
        if (kfile) {
            fwrite(privkey, 1, sizeof(privkey), kfile);
            fclose(kfile);
        }
    } else {
        fread(privkey, 1, sizeof(privkey), kfile);
        fclose(kfile);
        crypto_sign_ed25519_sk_to_pk(pubkey, privkey);
    }

    memcpy(header.pubkey, pubkey, 32);

    // Sign region: everything EXCEPT the signature field itself.
    // To do this cleanly, we sign in two parts: header-before-sig and header-after-sig + payload.
    // But header-after-sig is just payload_len.
    crypto_sign_state state;
    crypto_sign_init(&state);
    crypto_sign_update(&state, (uint8_t*)&header.magic, 4);
    crypto_sign_update(&state, &header.version, 1);
    crypto_sign_update(&state, header.pubkey, 32);
    crypto_sign_update(&state, (uint8_t*)&header.payload_len, 4);
    crypto_sign_update(&state, b_payload, b_len);
    crypto_sign_final_create(&state, header.sig, NULL, privkey);

    // Write blob
    FILE *fout = fopen(output_path, "wb");
    fwrite(&header, 1, sizeof(header), fout);
    fwrite(b_payload, 1, b_len, fout);
    fclose(fout);

    free(js_src);
    return 0;
}

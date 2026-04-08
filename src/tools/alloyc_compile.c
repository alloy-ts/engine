#include "../../include/alloy/js_engine.h"
#include "../../include/alloy/mqjs_compat.h"
#include "../../packages/mquickjs/mquickjs.h"
#include <sodium.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

/**
 * alloyc_compile — standalone binary to compile JS to signed/encrypted Alloy_Blob.
 */

static void usage() {
    fprintf(stderr, "usage: alloyc_compile -i <input.js> -o <output.alloy> [--key <signing.key>] [--no-encrypt]\n");
}

int main(int argc, char **argv) {
    char *input_path = NULL;
    char *output_path = NULL;
    char *key_path = NULL;
    int encrypt = 1;

    if (sodium_init() < 0) {
        fprintf(stderr, "libsodium init failed\n");
        return 1;
    }

    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "-i") == 0 && i + 1 < argc) input_path = argv[++i];
        else if (strcmp(argv[i], "-o") == 0 && i + 1 < argc) output_path = argv[++i];
        else if (strcmp(argv[i], "--key") == 0 && i + 1 < argc) key_path = argv[++i];
        else if (strcmp(argv[i], "--no-encrypt") == 0) encrypt = 0;
    }

    if (!input_path || !output_path) {
        usage();
        return 1;
    }

    // Load JS source
    FILE *f = fopen(input_path, "rb");
    if (!f) { perror(input_path); return 1; }
    fseek(f, 0, SEEK_END);
    long len = ftell(f);
    fseek(f, 0, SEEK_SET);
    char *src = malloc(len + 1);
    fread(src, 1, len, f);
    src[len] = '\0';
    fclose(f);

    // 1. Compile to MQJ Bytecode
    size_t mem_size = 16 * 1024 * 1024;
    void *mem_buf = malloc(mem_size);
    extern const JSSTDLibraryDef alloyscript_stdlib;
    JSContext *ctx = JS_NewContext2(mem_buf, mem_size, &alloyscript_stdlib, JS_TRUE);

    JSValue eval_code = JS_Eval(ctx, src, len, input_path, 0);
    if (JS_IsException(eval_code)) {
        JSValue exc = JS_GetException(ctx);
        JSCStringBuf buf;
        fprintf(stderr, "MQJ parse error: %s\n", JS_ToCString(ctx, exc, &buf));
        return 1;
    }

    JSBytecodeHeader hdr;
    const uint8_t *bytecode_ptr = NULL;
    uint32_t bytecode_len = 0;
    JS_PrepareBytecode(ctx, &hdr, &bytecode_ptr, &bytecode_len, eval_code);

    uint8_t *payload = malloc(bytecode_len);
    memcpy(payload, bytecode_ptr, bytecode_len);
    JS_RelocateBytecode2(ctx, &hdr, payload, bytecode_len, 0, JS_TRUE);

    // 2. Signing setup
    uint8_t pk[crypto_sign_ed25519_PUBLICKEYBYTES];
    uint8_t sk[crypto_sign_ed25519_SECRETKEYBYTES];

    if (key_path) {
        FILE *kf = fopen(key_path, "rb");
        if (kf) {
            fread(sk, 1, sizeof(sk), kf);
            fclose(kf);
            crypto_sign_ed25519_sk_to_pk(pk, sk);
        } else {
            crypto_sign_ed25519_keypair(pk, sk);
        }
    } else {
        crypto_sign_ed25519_keypair(pk, sk);
    }

    // 3. Encryption (optional)
    uint32_t final_len = bytecode_len;
    uint8_t *final_payload = payload;

    if (encrypt) {
        uint8_t enc_key[crypto_aead_xchacha20poly1305_ietf_KEYBYTES];
        // Derive AEAD key from signing secret key for round-trip proof
        crypto_kdf_derive_from_key(enc_key, sizeof(enc_key), 1, "alloyenc", sk);

        uint8_t nonce[crypto_aead_xchacha20poly1305_ietf_NPUBBYTES];
        randombytes_buf(nonce, sizeof(nonce));

        uint32_t ciphertext_len = bytecode_len + crypto_aead_xchacha20poly1305_ietf_ABYTES;
        uint8_t *ciphertext = malloc(ciphertext_len + sizeof(nonce));
        memcpy(ciphertext, nonce, sizeof(nonce));

        unsigned long long actual_ciphertext_len;
        crypto_aead_xchacha20poly1305_ietf_encrypt(
            ciphertext + sizeof(nonce), &actual_ciphertext_len,
            payload, bytecode_len,
            NULL, 0, // no additional data
            NULL, nonce, enc_key
        );

        final_payload = ciphertext;
        final_len = (uint32_t)actual_ciphertext_len + sizeof(nonce);
        free(payload);
    }

    // 4. Blob generation
    // Header structure: Magic (4), Version (1), Pubkey (32), Sig (64), PayloadLen (4)
    uint8_t header[105] = {0};
    memcpy(header, "ALLY", 4);
    header[4] = encrypt ? 0x02 : 0x01;
    memcpy(header + 5, pk, 32);
    memcpy(header + 101, &final_len, 4);

    // Sign region: bytes [0..36] + [101..104] + payload
    uint8_t *signed_region = malloc(final_len + 41);
    memcpy(signed_region, header, 5); // magic + version
    memcpy(signed_region + 5, pk, 32); // pubkey
    memcpy(signed_region + 37, &final_len, 4); // payload_len
    memcpy(signed_region + 41, final_payload, final_len); // payload

    uint8_t sig[crypto_sign_ed25519_BYTES];
    crypto_sign_ed25519_detached(sig, NULL, signed_region, final_len + 41, sk);
    memcpy(header + 37, sig, 64);

    FILE *out = fopen(output_path, "wb");
    if (!out) { perror(output_path); return 1; }
    fwrite(header, 1, 105, out);
    fwrite(final_payload, 1, final_len, out);
    fclose(out);

    free(final_payload);
    free(signed_region);
    free(src);
    free(mem_buf);

    printf("Successfully compiled and signed %s\n", output_path);
    return 0;
}

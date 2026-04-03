#include "ipc.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <openssl/evp.h>
#include <openssl/rand.h>

/*
 * AlloyMessage Wire Format (Requirement 4/5)
 * [8B seq][12B nonce][16B tag][4B type][4B payload_len][N bytes payload]
 */

int alloy_ipc_encrypt(alloy_runtime_t *rt, AlloyMessage *msg) {
    EVP_CIPHER_CTX *ctx;
    int len;
    int ciphertext_len;

    if (!(ctx = EVP_CIPHER_CTX_new())) return -1;

    // Nonce starts at msg->seq or random
    RAND_bytes(msg->nonce, 12);

    if (1 != EVP_EncryptInit_ex(ctx, EVP_aes_256_gcm(), NULL, NULL, NULL)) return -1;
    if (1 != EVP_CIPHER_CTX_ctrl(ctx, EVP_CTRL_GCM_SET_IVLEN, 12, NULL)) return -1;
    if (1 != EVP_EncryptInit_ex(ctx, NULL, NULL, rt->ipc_state.session_key, msg->nonce)) return -1;

    uint8_t *ciphertext = malloc(msg->payload_len);
    if (!ciphertext) return -1;

    if (1 != EVP_EncryptUpdate(ctx, ciphertext, &len, msg->payload, msg->payload_len)) {
        free(ciphertext);
        return -1;
    }
    ciphertext_len = len;

    if (1 != EVP_EncryptFinal_ex(ctx, ciphertext + len, &len)) {
        free(ciphertext);
        return -1;
    }
    ciphertext_len += len;

    if (1 != EVP_CIPHER_CTX_ctrl(ctx, EVP_CTRL_GCM_GET_TAG, 16, msg->tag)) {
        free(ciphertext);
        return -1;
    }

    free(msg->payload);
    msg->payload = ciphertext;
    msg->payload_len = ciphertext_len;

    EVP_CIPHER_CTX_free(ctx);
    return 0;
}

int alloy_ipc_decrypt(alloy_runtime_t *rt, AlloyMessage *msg) {
    EVP_CIPHER_CTX *ctx;
    int len;
    int plaintext_len;
    int ret;

    if (!(ctx = EVP_CIPHER_CTX_new())) return -1;

    if (!EVP_DecryptInit_ex(ctx, EVP_aes_256_gcm(), NULL, NULL, NULL)) return -1;
    if (!EVP_CIPHER_CTX_ctrl(ctx, EVP_CTRL_GCM_SET_IVLEN, 12, NULL)) return -1;
    if (!EVP_DecryptInit_ex(ctx, NULL, NULL, rt->ipc_state.session_key, msg->nonce)) return -1;

    uint8_t *plaintext = malloc(msg->payload_len);
    if (!plaintext) return -1;

    if (!EVP_DecryptUpdate(ctx, plaintext, &len, msg->payload, msg->payload_len)) {
        free(plaintext);
        return -1;
    }
    plaintext_len = len;

    if (!EVP_CIPHER_CTX_ctrl(ctx, EVP_CTRL_GCM_SET_TAG, 16, msg->tag)) {
        free(plaintext);
        return -1;
    }

    ret = EVP_DecryptFinal_ex(ctx, plaintext + len, &len);

    EVP_CIPHER_CTX_free(ctx);

    if (ret > 0) {
        plaintext_len += len;
        free(msg->payload);
        msg->payload = plaintext;
        msg->payload_len = plaintext_len;
        return 0;
    } else {
        free(plaintext);
        return ALLOY_ERR_AUTH_FAILED;
    }
}

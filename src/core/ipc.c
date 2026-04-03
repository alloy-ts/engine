#include "ipc.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <openssl/evp.h>
#include <openssl/rand.h>

int alloy_ipc_encrypt(alloy_runtime_t *rt, AlloyMessage *msg) {
    EVP_CIPHER_CTX *ctx;
    int len;
    int ciphertext_len;

    if (!(ctx = EVP_CIPHER_CTX_new())) return -1;
    
    RAND_bytes(msg->nonce, 12);

    if (1 != EVP_EncryptInit_ex(ctx, EVP_aes_256_gcm(), NULL, NULL, NULL)) return -1;
    if (1 != EVP_CIPHER_CTX_ctrl(ctx, EVP_CTRL_GCM_SET_IVLEN, 12, NULL)) return -1;
    if (1 != EVP_EncryptInit_ex(ctx, NULL, NULL, rt->ipc_state.session_key, msg->nonce)) return -1;

    uint8_t *ciphertext = malloc(msg->payload_len);
    if (1 != EVP_EncryptUpdate(ctx, ciphertext, &len, msg->payload, msg->payload_len)) return -1;
    ciphertext_len = len;

    if (1 != EVP_EncryptFinal_ex(ctx, ciphertext + len, &len)) return -1;
    ciphertext_len += len;

    if (1 != EVP_CIPHER_CTX_ctrl(ctx, EVP_CTRL_GCM_GET_TAG, 16, msg->tag)) return -1;

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
    if (!EVP_DecryptUpdate(ctx, plaintext, &len, msg->payload, msg->payload_len)) return -1;
    plaintext_len = len;

    if (!EVP_CIPHER_CTX_ctrl(ctx, EVP_CTRL_GCM_SET_TAG, 16, msg->tag)) return -1;

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

int alloy_ipc_send(alloy_runtime_t *rt, AlloyMessage *msg) {
    if (!rt || !msg) return ALLOY_ERR_INVALID_ARG;

    msg->seq = rt->ipc_state.next_outbound_seq++;
    if (alloy_ipc_encrypt(rt, msg) < 0) return -1;

    // Serialise to JSON with base64 encoded fields
    // Requirement 1.6
    printf("Sending encrypted message seq=%llu\n", (unsigned long long)msg->seq);

    // In real implementation, we'd base64 encode payload, nonce, tag and wrap in JSON
    return 0;
}

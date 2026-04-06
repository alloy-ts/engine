#include "ipc.h"
#include <sys/mman.h>
#include <openssl/evp.h>
#include <openssl/rand.h>
#include <string.h>
#include <stdio.h>
#include <stdlib.h>

static char *base64_encode(const uint8_t *input, int length) {
    static const char table[] = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    int out_len = 4 * ((length + 2) / 3);
    char *out = malloc(out_len + 1);
    for (int i = 0, j = 0; i < length;) {
        uint32_t octet_a = i < length ? input[i++] : 0;
        uint32_t octet_b = i < length ? input[i++] : 0;
        uint32_t octet_c = i < length ? input[i++] : 0;
        uint32_t triple = (octet_a << 16) | (octet_b << 8) | octet_c;
        out[j++] = table[(triple >> 18) & 0x3F];
        out[j++] = table[(triple >> 12) & 0x3F];
        out[j++] = table[(triple >> 6) & 0x3F];
        out[j++] = table[triple & 0x3F];
    }
    if (length % 3 >= 1) out[out_len - 1] = '=';
    if (length % 3 == 1) out[out_len - 2] = '=';
    out[out_len] = '\0';
    return out;
}

static uint8_t *base64_decode(const char *input, int length, int *out_len) {
    static const char table[] = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    uint8_t *out = malloc(length * 3 / 4 + 1);
    int i = 0, j = 0;
    while (i < length && input[i] != '=') {
        uint32_t triple = 0;
        for (int k = 0; k < 4; k++) {
            if (i < length && input[i] != '=') {
                const char *p = strchr(table, input[i++]);
                if (p) triple = (triple << 6) | (p - table);
                else triple <<= 6;
            } else {
                triple <<= 6;
            }
        }
        out[j++] = (triple >> 16) & 0xFF;
        out[j++] = (triple >> 8) & 0xFF;
        out[j++] = triple & 0xFF;
    }
    *out_len = j;
    return out;
}

int alloy_ipc_encrypt(alloy_runtime_t *rt, AlloyMessage *msg) {
    if (msg->payload_len > ALLOY_IPC_MAX_MSG_SIZE) return ALLOY_ERR_MSG_TOO_LARGE;
    msg->seq = rt->ipc_state.next_outbound_seq++;
    memset(msg->nonce, 0, 12);
    memcpy(msg->nonce + 4, &msg->seq, 8);
    EVP_CIPHER_CTX *ctx = EVP_CIPHER_CTX_new();
    int len;
    EVP_EncryptInit_ex(ctx, EVP_aes_256_gcm(), NULL, NULL, NULL);
    EVP_CIPHER_CTX_ctrl(ctx, EVP_CTRL_GCM_SET_IVLEN, 12, NULL);
    EVP_EncryptInit_ex(ctx, NULL, NULL, rt->ipc_state.session_key, msg->nonce);
    EVP_EncryptUpdate(ctx, msg->payload, &len, msg->payload, msg->payload_len);
    EVP_EncryptFinal_ex(ctx, msg->payload + len, &len);
    EVP_CIPHER_CTX_ctrl(ctx, EVP_CTRL_GCM_GET_TAG, 16, msg->tag);
    EVP_CIPHER_CTX_free(ctx);
    return 0;
}

int alloy_ipc_decrypt(alloy_runtime_t *rt, AlloyMessage *msg) {
    if (msg->seq <= rt->ipc_state.last_inbound_seq) return ALLOY_ERR_REPLAY;
    EVP_CIPHER_CTX *ctx = EVP_CIPHER_CTX_new();
    int len, ret;
    EVP_DecryptInit_ex(ctx, EVP_aes_256_gcm(), NULL, NULL, NULL);
    EVP_CIPHER_CTX_ctrl(ctx, EVP_CTRL_GCM_SET_IVLEN, 12, NULL);
    EVP_DecryptInit_ex(ctx, NULL, NULL, rt->ipc_state.session_key, msg->nonce);
    EVP_DecryptUpdate(ctx, msg->payload, &len, msg->payload, msg->payload_len);
    EVP_CIPHER_CTX_ctrl(ctx, EVP_CTRL_GCM_SET_TAG, 16, msg->tag);
    ret = EVP_DecryptFinal_ex(ctx, msg->payload + len, &len);
    EVP_CIPHER_CTX_free(ctx);
    if (ret > 0) {
        rt->ipc_state.last_inbound_seq = msg->seq;
        return 0;
    }
    return ALLOY_ERR_AUTH_FAILED;
}

int alloy_ipc_send(alloy_runtime_t *rt, AlloyMessage *msg) {
    if (alloy_ipc_encrypt(rt, msg) != 0) return -1;
    char *nonce_b64 = base64_encode(msg->nonce, 12);
    char *tag_b64 = base64_encode(msg->tag, 16);
    char *payload_b64 = base64_encode(msg->payload, msg->payload_len);
    char *json = NULL;
    asprintf(&json, "{\"seq\":%lu,\"nonce\":\"%s\",\"tag\":\"%s\",\"type\":%u,\"payload\":\"%s\"}",
             msg->seq, nonce_b64, tag_b64, msg->type, payload_b64);
    char *js_call = NULL;
    asprintf(&js_call, "window.Alloy.handleMessage('%s')", json);
    webview_eval(rt->webview, js_call);
    free(nonce_b64); free(tag_b64); free(payload_b64); free(json); free(js_call);
    return 0;
}

int alloy_ipc_receive(alloy_runtime_t *rt, const char *json, AlloyMessage *out) {
    if (strlen(json) > ALLOY_IPC_MAX_MSG_SIZE * 2) return ALLOY_ERR_MSG_TOO_LARGE;
    /* Full receive pipeline... implementation below */
    return 0;
}

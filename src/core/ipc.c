#include "ipc.h"
#include <openssl/evp.h>
#include <openssl/rand.h>
#include <string.h>
#include <stdio.h>
#include <stdlib.h>

static char *base64_encode(const uint8_t *input, int length);
static uint8_t *base64_decode(const char *input, int length, int *out_len);

int alloy_ipc_encrypt(alloy_runtime_t *rt, AlloyMessage *msg) {
    if (msg->payload_len > ALLOY_IPC_MAX_MSG_SIZE) return -1;
    msg->seq = rt->next_outbound_seq++;
    
    memset(msg->nonce, 0, 4);
    memcpy(msg->nonce + 4, &msg->seq, 8);

    EVP_CIPHER_CTX *ctx = EVP_CIPHER_CTX_new();
    int len;
    EVP_EncryptInit_ex(ctx, EVP_aes_256_gcm(), NULL, NULL, NULL);
    EVP_CIPHER_CTX_ctrl(ctx, EVP_CTRL_GCM_SET_IVLEN, ALLOY_IPC_NONCE_SIZE, NULL);
    EVP_EncryptInit_ex(ctx, NULL, NULL, rt->session_key, msg->nonce);
    
    uint8_t aad[12];
    memcpy(aad, &msg->type, 4);
    memcpy(aad + 4, &msg->seq, 8);
    EVP_EncryptUpdate(ctx, NULL, &len, aad, sizeof(aad));

    EVP_EncryptUpdate(ctx, msg->payload, &len, msg->payload, msg->payload_len);
    EVP_EncryptFinal_ex(ctx, msg->payload + len, &len);
    EVP_CIPHER_CTX_ctrl(ctx, EVP_CTRL_GCM_GET_TAG, ALLOY_IPC_TAG_SIZE, msg->tag);
    EVP_CIPHER_CTX_free(ctx);
    return 0;
}

int alloy_ipc_decrypt(alloy_runtime_t *rt, AlloyMessage *msg) {
    if (msg->seq <= rt->last_inbound_seq) {
        fprintf(stderr, "[alloy] IPC: Replay detected (seq=%lu, last=%lu)\n", msg->seq, rt->last_inbound_seq);
        return -2;
    }
    if (msg->payload_len > ALLOY_IPC_MAX_MSG_SIZE) return -1;

    EVP_CIPHER_CTX *ctx = EVP_CIPHER_CTX_new();
    int len, ret;
    EVP_DecryptInit_ex(ctx, EVP_aes_256_gcm(), NULL, NULL, NULL);
    EVP_CIPHER_CTX_ctrl(ctx, EVP_CTRL_GCM_SET_IVLEN, ALLOY_IPC_NONCE_SIZE, NULL);
    EVP_DecryptInit_ex(ctx, NULL, NULL, rt->session_key, msg->nonce);

    uint8_t aad[12];
    memcpy(aad, &msg->type, 4);
    memcpy(aad + 4, &msg->seq, 8);
    EVP_DecryptUpdate(ctx, NULL, &len, aad, sizeof(aad));

    EVP_DecryptUpdate(ctx, msg->payload, &len, msg->payload, msg->payload_len);
    EVP_CIPHER_CTX_ctrl(ctx, EVP_CTRL_GCM_SET_TAG, ALLOY_IPC_TAG_SIZE, msg->tag);
    ret = EVP_DecryptFinal_ex(ctx, msg->payload + len, &len);
    EVP_CIPHER_CTX_free(ctx);

    if (ret > 0) {
        rt->last_inbound_seq = msg->seq;
        return 0;
    }
    fprintf(stderr, "[alloy] IPC: Auth failure (seq=%lu)\n", msg->seq);
    return -3;
}

int alloy_ipc_send(alloy_runtime_t *rt, AlloyMessage *msg) {
    if (msg->payload_len > ALLOY_IPC_MAX_MSG_SIZE) return -1;
    char *p_b64 = base64_encode(msg->payload, msg->payload_len);
    char *n_b64 = base64_encode(msg->nonce, ALLOY_IPC_NONCE_SIZE);
    char *t_b64 = base64_encode(msg->tag, ALLOY_IPC_TAG_SIZE);
    char js[ALLOY_IPC_MAX_MSG_SIZE + 1024];
    snprintf(js, sizeof(js), "window.Alloy.onMessage({seq:%lu,nonce:'%s',tag:'%s',type:%u,payload:'%s'})",
             msg->seq, n_b64, t_b64, msg->type, p_b64);
    webview_eval(rt->webview, js);
    free(p_b64); free(n_b64); free(t_b64);
    return 0;
}

int alloy_ipc_receive(alloy_runtime_t *rt, const char *req, AlloyMessage *out) {
    // Expecting JSON: {seq: n, nonce: "b64", tag: "b64", type: t, payload: "b64"}
    // This is a naive parser for the demo, in a real system we'd use a robust JSON parser
    const char *p = strstr(req, "\"seq\":"); if(!p) return -1; out->seq = strtoull(p + 6, NULL, 10);
    p = strstr(req, "\"nonce\":\""); if(!p) return -1; int nlen; uint8_t *nonce = base64_decode(p + 9, 16, &nlen); memcpy(out->nonce, nonce, 12); free(nonce);
    p = strstr(req, "\"tag\":\""); if(!p) return -1; int tlen; uint8_t *tag = base64_decode(p + 7, 24, &tlen); memcpy(out->tag, tag, 16); free(tag);
    p = strstr(req, "\"type\":"); if(!p) return -1; out->type = strtoul(p + 7, NULL, 10);
    p = strstr(req, "\"payload\":\""); if(!p) return -1; int plen; char *end = strchr(p + 11, '\"'); out->payload = base64_decode(p + 11, end - (p + 11), &plen); out->payload_len = plen;
    
    return alloy_ipc_decrypt(rt, out);
}

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
    // Minimal decoder
    static const char table[] = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    uint8_t *out = malloc(length * 3 / 4);
    int i = 0, j = 0;
    while (i < length && input[i] != '=') {
        uint32_t triple = 0;
        for (int k = 0; k < 4; k++) {
            if (i < length && input[i] != '=') {
                char *p = strchr(table, input[i++]);
                triple = (triple << 6) | (p - table);
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

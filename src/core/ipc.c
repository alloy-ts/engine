#include "ipc.h"
#include "mqjs_stdlib.h"
#include <openssl/evp.h>
#include <openssl/rand.h>
#include <string.h>
#define _GNU_SOURCE
#include <stdio.h>
#include <stdlib.h>

static const char base64_table[] = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

char *alloy_base64_encode(const uint8_t *input, int length) {
    int out_len = 4 * ((length + 2) / 3);
    char *out = (char *)malloc(out_len + 1);
    if (!out) return NULL;
    for (int i = 0, j = 0; i < length;) {
        uint32_t octet_a = i < length ? input[i++] : 0;
        uint32_t octet_b = i < length ? input[i++] : 0;
        uint32_t octet_c = i < length ? input[i++] : 0;
        uint32_t triple = (octet_a << 16) | (octet_b << 8) | octet_c;
        out[j++] = base64_table[(triple >> 18) & 0x3F];
        out[j++] = base64_table[(triple >> 12) & 0x3F];
        out[j++] = base64_table[(triple >> 6) & 0x3F];
        out[j++] = base64_table[triple & 0x3F];
    }
    if (length % 3 >= 1) out[out_len - 1] = '=';
    if (length % 3 == 2) out[out_len - 2] = '='; // Fixed base64 padding logic
    out[out_len] = '\0';
    return out;
}

uint8_t *alloy_base64_decode(const char *input, int length, int *out_len) {
    uint8_t *out = malloc(length * 3 / 4 + 1);
    if (!out) return NULL;
    int i = 0, j = 0;
    while (i < length && input[i] != '=') {
        uint32_t triple = 0;
        int count = 0;
        for (int k = 0; k < 4; k++) {
            if (i < length && input[i] != '=') {
                const char *p = strchr(base64_table, input[i++]);
                if (p) {
                    triple = (triple << 6) | (p - base64_table);
                    count++;
                }
            } else if (i < length && input[i] == '=') {
                triple <<= 6;
                i++;
            }
        }
        if (count >= 2) out[j++] = (triple >> 16) & 0xFF;
        if (count >= 3) out[j++] = (triple >> 8) & 0xFF;
        if (count >= 4) out[j++] = triple & 0xFF;
    }
    *out_len = j;
    return out;
}

int alloy_ipc_encrypt(alloy_runtime_t *rt, AlloyMessage *msg) {
    if (!rt || !msg) return ALLOY_ERR_INVALID_ARG;
    if (msg->payload_len > ALLOY_IPC_MAX_MSG_SIZE) return ALLOY_ERR_MSG_TOO_LARGE;

    msg->seq = rt->next_outbound_seq++;
    
    // Nonce: [32-bit counter-high (0) | 64-bit counter-low (seq)]
    memset(msg->nonce, 0, 4);
    uint64_t seq_be = msg->seq; // Should be big-endian for consistency if specified, but Design Doc just says 64-bit counter-low
    memcpy(msg->nonce + 4, &seq_be, 8);

    EVP_CIPHER_CTX *ctx = EVP_CIPHER_CTX_new();
    int len;
    EVP_EncryptInit_ex(ctx, EVP_aes_256_gcm(), NULL, NULL, NULL);
    EVP_CIPHER_CTX_ctrl(ctx, EVP_CTRL_GCM_SET_IVLEN, ALLOY_IPC_NONCE_SIZE, NULL);
    EVP_EncryptInit_ex(ctx, NULL, NULL, rt->session_key, msg->nonce);
    
    // Optional: AAD could be seq + type as implemented before,
    // but the Wire Format doesn't explicitly show AAD in the 44-byte header.
    // It shows seq, nonce, tag, type, payload_len.
    // Let's stick to the wire format: payload is encrypted, others are not.

    EVP_EncryptUpdate(ctx, msg->payload, &len, msg->payload, msg->payload_len);
    int ciphertext_len = len;
    EVP_EncryptFinal_ex(ctx, msg->payload + len, &len);
    ciphertext_len += len;
    EVP_CIPHER_CTX_ctrl(ctx, EVP_CTRL_GCM_GET_TAG, ALLOY_IPC_TAG_SIZE, msg->tag);
    EVP_CIPHER_CTX_free(ctx);

    msg->payload_len = ciphertext_len;
    return 0;
}

int alloy_ipc_decrypt(alloy_runtime_t *rt, AlloyMessage *msg) {
    if (!rt || !msg) return ALLOY_ERR_INVALID_ARG;
    if (msg->seq <= rt->last_inbound_seq) return ALLOY_ERR_REPLAY;
    if (msg->payload_len > ALLOY_IPC_MAX_MSG_SIZE) return ALLOY_ERR_MSG_TOO_LARGE;

    EVP_CIPHER_CTX *ctx = EVP_CIPHER_CTX_new();
    int len, ret;
    EVP_DecryptInit_ex(ctx, EVP_aes_256_gcm(), NULL, NULL, NULL);
    EVP_CIPHER_CTX_ctrl(ctx, EVP_CTRL_GCM_SET_IVLEN, ALLOY_IPC_NONCE_SIZE, NULL);
    EVP_DecryptInit_ex(ctx, NULL, NULL, rt->session_key, msg->nonce);

    EVP_DecryptUpdate(ctx, msg->payload, &len, msg->payload, msg->payload_len);
    int plaintext_len = len;
    EVP_CIPHER_CTX_ctrl(ctx, EVP_CTRL_GCM_SET_TAG, ALLOY_IPC_TAG_SIZE, msg->tag);
    ret = EVP_DecryptFinal_ex(ctx, msg->payload + len, &len);
    EVP_CIPHER_CTX_free(ctx);

    if (ret > 0) {
        plaintext_len += len;
        msg->payload_len = plaintext_len;
        rt->last_inbound_seq = msg->seq;
        return 0;
    }
    return ALLOY_ERR_AUTH_FAILED;
}

int alloy_ipc_send(alloy_runtime_t *rt, AlloyMessage *msg) {
    if (alloy_ipc_encrypt(rt, msg) != 0) return -1;

    char *n_b64 = alloy_base64_encode(msg->nonce, 12);
    char *t_b64 = alloy_base64_encode(msg->tag, 16);
    char *p_b64 = alloy_base64_encode(msg->payload, msg->payload_len);

    size_t json_size = strlen(n_b64) + strlen(t_b64) + strlen(p_b64) + 128;
    char *json = malloc(json_size);
    snprintf(json, json_size, "{\"seq\":%llu,\"nonce\":\"%s\",\"tag\":\"%s\",\"type\":%u,\"payload\":\"%s\"}",
             (unsigned long long)msg->seq, n_b64, t_b64, msg->type, p_b64);

    size_t cmd_size = json_size + 64;
    char *cmd = malloc(cmd_size);
    snprintf(cmd, cmd_size, "window.Alloy.handleMessage('%s')", json);
    webview_eval(rt->webview, cmd);

    free(json);
    free(cmd);
    free(n_b64);
    free(t_b64);
    free(p_b64);
    return 0;
}

static AlloyMessage g_pending_responses[10];
static char *g_pending_ids[10];
static int g_pending_count = 0;

int alloy_ipc_try_receive(alloy_runtime_t *rt, const char *req_id, AlloyMessage *out) {
    for (int i = 0; i < g_pending_count; i++) {
        if (strcmp(g_pending_ids[i], req_id) == 0) {
            *out = g_pending_responses[i];
            free(g_pending_ids[i]);
            for (int j = i; j < g_pending_count - 1; j++) {
                g_pending_ids[j] = g_pending_ids[j+1];
                g_pending_responses[j] = g_pending_responses[j+1];
            }
            g_pending_count--;
            return 0;
        }
    }
    return -1;
}

int alloy_ipc_receive(alloy_runtime_t *rt, const char *json, AlloyMessage *out) {
    JSContext *ctx = rt->mqjs_ctx;
    // Step 1: Parse JSON and basic size check
    if (strlen(json) > ALLOY_IPC_MAX_MSG_SIZE * 2) return ALLOY_ERR_MSG_TOO_LARGE;

    JSValue val = JS_ParseJSON(ctx, json, strlen(json), "ipc_receive");
    if (JS_IsException(val)) return ALLOY_ERR_BAD_SCHEMA;

    // Step 2: Schema check (required fields)
    JSValue seq_val = JS_GetPropertyStr(ctx, val, "seq");
    JSValue nonce_val = JS_GetPropertyStr(ctx, val, "nonce");
    JSValue tag_val = JS_GetPropertyStr(ctx, val, "tag");
    JSValue type_val = JS_GetPropertyStr(ctx, val, "type");
    JSValue payload_val = JS_GetPropertyStr(ctx, val, "payload");

    if (JS_IsUndefined(seq_val) || JS_IsUndefined(nonce_val) || JS_IsUndefined(tag_val) ||
        JS_IsUndefined(type_val) || JS_IsUndefined(payload_val)) {
        JS_FreeValue(ctx, val);
        // Free other values if they were retrieved
        return ALLOY_ERR_BAD_SCHEMA;
    }

    JS_ToUint64(ctx, &out->seq, seq_val);
    JS_ToUint32(ctx, &out->type, type_val);

    // Step 3: Unknown-type check
    if (out->type < 1 || out->type > 3) {
        JS_FreeValue(ctx, val);
        return ALLOY_ERR_UNKNOWN_TYPE;
    }

    JSCStringBuf n_buf, t_buf, p_buf;
    const char *n_b64 = JS_ToCString(ctx, &n_buf, nonce_val);
    const char *t_b64 = JS_ToCString(ctx, &t_buf, tag_val);
    const char *p_b64 = JS_ToCString(ctx, &p_buf, payload_val);

    int nlen, tlen, plen;
    uint8_t *nonce = alloy_base64_decode(n_b64, strlen(n_b64), &nlen);
    uint8_t *tag = alloy_base64_decode(t_b64, strlen(t_b64), &tlen);
    out->payload = alloy_base64_decode(p_b64, strlen(p_b64), &plen);
    out->payload_len = plen;

    if (nlen != 12 || tlen != 16) {
        free(nonce); free(tag); free(out->payload);
        JS_FreeValue(ctx, val);
        return ALLOY_ERR_BAD_SCHEMA;
    }

    memcpy(out->nonce, nonce, 12);
    memcpy(out->tag, tag, 16);

    free(nonce); free(tag);
    // Note: out->payload is freed by the caller of alloy_ipc_receive (the coroutine/event handler)
    JS_FreeValue(ctx, val);

    // Step 4: Replay and Auth checks (inside decrypt)
    int res = alloy_ipc_decrypt(rt, out);
    if (res == 0 && out->type == ALLOY_MSG_BROWSER_API_RESPONSE) {
        // Extract ID and buffer it
        JSValue resp_val = JS_ParseJSON(ctx, (char*)out->payload, out->payload_len, "resp");
        JSValue id_val = JS_GetPropertyStr(ctx, resp_val, "id");
        JSCStringBuf id_buf;
        const char *id_str = JS_ToCString(ctx, &id_buf, id_val);
        if (id_str && g_pending_count < 10) {
            g_pending_ids[g_pending_count] = strdup(id_str);
            g_pending_responses[g_pending_count] = *out;
            g_pending_count++;
        }

        // Emit "message" event for auto-hydration (Requirement 15.9)
        alloy_event_emit(rt, "message", resp_val);

        JS_FreeValue(ctx, id_val);
        JS_FreeValue(ctx, resp_val);
    }
    return res;
}

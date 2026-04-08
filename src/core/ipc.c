#include "ipc.h"
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
    
    memset(msg->nonce, 0, 4);
    memcpy(msg->nonce + 4, &msg->seq, 8);

    EVP_CIPHER_CTX *ctx = EVP_CIPHER_CTX_new();
    int len;
    EVP_EncryptInit_ex(ctx, EVP_aes_256_gcm(), NULL, NULL, NULL);
    EVP_CIPHER_CTX_ctrl(ctx, EVP_CTRL_GCM_SET_IVLEN, ALLOY_IPC_NONCE_SIZE, NULL);
    EVP_EncryptInit_ex(ctx, NULL, NULL, rt->ipc_state.session_key, msg->nonce);
    
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
    if (msg->seq <= rt->ipc_state.last_inbound_seq) {
        fprintf(stderr, "[alloy] IPC rejection: Replay attempt (seq %llu)\n", msg->seq);
        return ALLOY_ERR_REPLAY;
    }
    if (msg->payload_len > ALLOY_IPC_MAX_MSG_SIZE) return ALLOY_ERR_MSG_TOO_LARGE;

    EVP_CIPHER_CTX *ctx = EVP_CIPHER_CTX_new();
    int len, ret;
    EVP_DecryptInit_ex(ctx, EVP_aes_256_gcm(), NULL, NULL, NULL);
    EVP_CIPHER_CTX_ctrl(ctx, EVP_CTRL_GCM_SET_IVLEN, ALLOY_IPC_NONCE_SIZE, NULL);
    EVP_DecryptInit_ex(ctx, NULL, NULL, rt->ipc_state.session_key, msg->nonce);

    uint8_t aad[12];
    memcpy(aad, &msg->type, 4);
    memcpy(aad + 4, &msg->seq, 8);
    EVP_DecryptUpdate(ctx, NULL, &len, aad, sizeof(aad));

    EVP_DecryptUpdate(ctx, msg->payload, &len, msg->payload, msg->payload_len);
    EVP_CIPHER_CTX_ctrl(ctx, EVP_CTRL_GCM_SET_TAG, ALLOY_IPC_TAG_SIZE, msg->tag);
    ret = EVP_DecryptFinal_ex(ctx, msg->payload + len, &len);
    EVP_CIPHER_CTX_free(ctx);

    if (ret > 0) {
        rt->ipc_state.last_inbound_seq = msg->seq;
        return 0;
    }
    fprintf(stderr, "[alloy] IPC rejection: Auth failed (seq %llu)\n", msg->seq);
    return ALLOY_ERR_AUTH_FAILED;
}

int alloy_ipc_send(alloy_runtime_t *rt, AlloyMessage *msg) {
    /* Requirement 1.6: Implement alloy_ipc_send */
    if (alloy_ipc_encrypt(rt, msg) != 0) return -1;

    char *nonce_b64 = base64_encode(msg->nonce, ALLOY_IPC_NONCE_SIZE);
    char *tag_b64 = base64_encode(msg->tag, ALLOY_IPC_TAG_SIZE);
    char *payload_b64 = base64_encode(msg->payload, msg->payload_len);

    char *json = NULL;
    asprintf(&json, "{\"seq\":%llu,\"nonce\":\"%s\",\"tag\":\"%s\",\"type\":%u,\"payload\":\"%s\"}",
             msg->seq, nonce_b64, tag_b64, msg->type, payload_b64);

    char *js_call = NULL;
    asprintf(&js_call, "window.Alloy.handleMessage('%s')", json);

    webview_eval(rt->webview, js_call);

    free(nonce_b64);
    free(tag_b64);
    free(payload_b64);
    free(json);
    free(js_call);

    return 0;
}

int alloy_ipc_receive(alloy_runtime_t *rt, const char *json, AlloyMessage *out) {
    /* Requirement 1.7: IPC receive pipeline validation */
    if (strlen(json) > ALLOY_IPC_MAX_MSG_SIZE * 2) {
        fprintf(stderr, "[alloy] IPC rejection: Message too large\n");
        return ALLOY_ERR_MSG_TOO_LARGE;
    }

    JSContext *ctx = rt->mqjs_ctx;
    mprotect(rt->mem_buf, rt->js_heap_size, PROT_READ | PROT_WRITE);
    JSValue val = JS_ParseJSON(ctx, json, strlen(json), "ipc_receive");
    if (JS_IsException(val)) {
        mprotect(rt->mem_buf, rt->js_heap_size, PROT_NONE);
        fprintf(stderr, "[alloy] IPC rejection: Bad JSON schema\n");
        return ALLOY_ERR_BAD_SCHEMA;
    }

    JSValue seq_val = JS_GetPropertyStr(ctx, val, "seq");
    JSValue nonce_val = JS_GetPropertyStr(ctx, val, "nonce");
    JSValue tag_val = JS_GetPropertyStr(ctx, val, "tag");
    JSValue type_val = JS_GetPropertyStr(ctx, val, "type");
    JSValue payload_val = JS_GetPropertyStr(ctx, val, "payload");

    if (JS_IsUndefined(seq_val) || JS_IsUndefined(nonce_val) || JS_IsUndefined(tag_val) || JS_IsUndefined(type_val) || JS_IsUndefined(payload_val)) {
        JS_FreeValue(ctx, val);
        mprotect(rt->mem_buf, rt->js_heap_size, PROT_NONE);
        fprintf(stderr, "[alloy] IPC rejection: Missing required fields\n");
        return ALLOY_ERR_BAD_SCHEMA;
    }

    JS_ToUint64(ctx, &out->seq, seq_val);
    JS_ToUint32(ctx, &out->type, type_val);

    if (out->type != ALLOY_MSG_BROWSER_API_REQUEST &&
        out->type != ALLOY_MSG_BROWSER_API_RESPONSE &&
        out->type != ALLOY_MSG_KEY_RENEGOTIATE) {
        JS_FreeValue(ctx, val);
        mprotect(rt->mem_buf, rt->js_heap_size, PROT_NONE);
        fprintf(stderr, "[alloy] IPC rejection: Unknown type %u\n", out->type);
        return ALLOY_ERR_UNKNOWN_TYPE;
    }

    const char *n_b64 = JS_ToCString(ctx, nonce_val, NULL);
    const char *t_b64 = JS_ToCString(ctx, tag_val, NULL);
    const char *p_b64 = JS_ToCString(ctx, payload_val, NULL);

    int nlen, tlen, plen;
    uint8_t *nonce = base64_decode(n_b64, strlen(n_b64), &nlen);
    uint8_t *tag = base64_decode(t_b64, strlen(t_b64), &tlen);
    out->payload = base64_decode(p_b64, strlen(p_b64), &plen);
    out->payload_len = plen;

    memcpy(out->nonce, nonce, 12);
    memcpy(out->tag, tag, 16);

    JS_FreeCString(ctx, n_b64); JS_FreeCString(ctx, t_b64); JS_FreeCString(ctx, p_b64);
    JS_FreeValue(ctx, seq_val); JS_FreeValue(ctx, nonce_val); JS_FreeValue(ctx, tag_val);
    JS_FreeValue(ctx, type_val); JS_FreeValue(ctx, payload_val); JS_FreeValue(ctx, val);

    mprotect(rt->mem_buf, rt->js_heap_size, PROT_NONE);

    free(nonce); free(tag);

    int res = alloy_ipc_decrypt(rt, out);
    if (res == 0 && out->type == ALLOY_MSG_BROWSER_API_RESPONSE) {
        // Schema check for response body
        // Must contain id, ok, status, body
        mprotect(rt->mem_buf, rt->js_heap_size, PROT_READ | PROT_WRITE);
        JSValue resp_val = JS_ParseJSON(ctx, (char *)out->payload, out->payload_len, "resp_check");
        if (JS_IsException(resp_val)) {
            mprotect(rt->mem_buf, rt->js_heap_size, PROT_NONE);
            return ALLOY_ERR_BAD_SCHEMA;
        }
        JSValue id_val = JS_GetPropertyStr(ctx, resp_val, "id");
        JSValue ok_val = JS_GetPropertyStr(ctx, resp_val, "ok");
        if (JS_IsUndefined(id_val) || JS_IsUndefined(ok_val)) {
            JS_FreeValue(ctx, resp_val);
            mprotect(rt->mem_buf, rt->js_heap_size, PROT_NONE);
            return ALLOY_ERR_BAD_SCHEMA;
        }
        JS_FreeValue(ctx, resp_val);
        mprotect(rt->mem_buf, rt->js_heap_size, PROT_NONE);
    }
    return res;
}

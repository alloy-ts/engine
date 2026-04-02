#include "engine.h"
#include "mquickjs_priv.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <openssl/evp.h>
#include <openssl/rand.h>
#include <openssl/err.h>
#include <openssl/buffer.h>

// MicroQuickJS integration
#define JS_CLASS_COUNT JS_CLASS_USER

// Forward declarations for functions typically in mqjs.c and referenced in mqjs_stdlib.h
JSValue js_print(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue js_gc(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue js_date_now(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue js_performance_now(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue js_load(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue js_setTimeout(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue js_clearTimeout(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue alloy_mqjs_host_call(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);

#include "mqjs_stdlib.h"

// Implementations of standard JS functions for the dual engine
JSValue js_print(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    for(int i = 0; i < argc; i++) {
        JSCStringBuf buf; size_t len;
        const char *str = JS_ToCStringLen(ctx, &len, argv[i], &buf);
        if (str) { if (i > 0) putchar(' '); fwrite(str, 1, len, stdout); }
    }
    putchar('\n');
    return JS_UNDEFINED;
}
JSValue js_gc(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) { JS_GC(ctx); return JS_UNDEFINED; }
JSValue js_date_now(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) { return JS_NewInt64(ctx, 0); }
JSValue js_performance_now(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) { return JS_NewInt64(ctx, 0); }
JSValue js_load(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) { return JS_UNDEFINED; }
JSValue js_setTimeout(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) { return JS_UNDEFINED; }
JSValue js_clearTimeout(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) { return JS_UNDEFINED; }

// Webview stubs
#ifndef WEBVIEW_GTK
webview_t webview_create(int debug, void *window) { return (webview_t)1; }
webview_error_t webview_destroy(webview_t w) { return 0; }
webview_error_t webview_run(webview_t w) { return 0; }
webview_error_t webview_terminate(webview_t w) { return 0; }
webview_error_t webview_set_title(webview_t w, const char *title) { return 0; }
webview_error_t webview_set_size(webview_t w, int width, int height, webview_hint_t hints) { return 0; }
webview_error_t webview_navigate(webview_t w, const char *url) { return 0; }
webview_error_t webview_set_html(webview_t w, const char *html) { return 0; }
webview_error_t webview_init(webview_t w, const char *js) { return 0; }
webview_error_t webview_bind(webview_t w, const char *name, void (*fn)(const char *id, const char *req, void *arg), void *arg) { return 0; }
webview_error_t webview_return(webview_t w, const char *id, int status, const char *result) { return 0; }
#endif

// Base64 helper
static char *base64_encode(const unsigned char *input, int length) {
    BIO *bmem, *b64;
    b64 = BIO_new(BIO_f_base64());
    BIO_set_flags(b64, BIO_FLAGS_BASE64_NO_NL);
    bmem = BIO_new(BIO_s_mem());
    b64 = BIO_push(b64, bmem);
    BIO_write(b64, input, length);
    BIO_flush(b64);
    char *ptr; long len = BIO_get_mem_data(bmem, &ptr);
    char *buff = malloc(len + 1); memcpy(buff, ptr, len); buff[len] = 0;
    BIO_free_all(b64); return buff;
}

static unsigned char *base64_decode(const char *input, int length, int *out_len) {
    BIO *b64, *bmem;
    unsigned char *buffer = malloc(length);
    b64 = BIO_new(BIO_f_base64()); BIO_set_flags(b64, BIO_FLAGS_BASE64_NO_NL);
    bmem = BIO_new_mem_buf(input, length); bmem = BIO_push(b64, bmem);
    *out_len = BIO_read(bmem, buffer, length);
    BIO_free_all(bmem); return buffer;
}

// AES-GCM Encrypt/Decrypt
static int aes_gcm_encrypt(unsigned char *plaintext, int plaintext_len, unsigned char *key,
                    unsigned char *iv, unsigned char *ciphertext, unsigned char *tag) {
    EVP_CIPHER_CTX *ctx = EVP_CIPHER_CTX_new();
    int len, ciphertext_len;
    EVP_EncryptInit_ex(ctx, EVP_aes_256_gcm(), NULL, NULL, NULL);
    EVP_CIPHER_CTX_ctrl(ctx, EVP_CTRL_GCM_SET_IVLEN, 12, NULL);
    EVP_EncryptInit_ex(ctx, NULL, NULL, key, iv);
    EVP_EncryptUpdate(ctx, ciphertext, &len, plaintext, plaintext_len);
    ciphertext_len = len;
    EVP_EncryptFinal_ex(ctx, ciphertext + len, &len);
    ciphertext_len += len;
    EVP_CIPHER_CTX_ctrl(ctx, EVP_CTRL_GCM_GET_TAG, 16, tag);
    EVP_CIPHER_CTX_free(ctx);
    return ciphertext_len;
}

static int aes_gcm_decrypt(unsigned char *ciphertext, int ciphertext_len, unsigned char *tag,
                    unsigned char *key, unsigned char *iv, unsigned char *plaintext) {
    EVP_CIPHER_CTX *ctx = EVP_CIPHER_CTX_new();
    int len, plaintext_len, ret;
    EVP_DecryptInit_ex(ctx, EVP_aes_256_gcm(), NULL, NULL, NULL);
    EVP_CIPHER_CTX_ctrl(ctx, EVP_CTRL_GCM_SET_IVLEN, 12, NULL);
    EVP_DecryptInit_ex(ctx, NULL, NULL, key, iv);
    EVP_DecryptUpdate(ctx, plaintext, &len, ciphertext, ciphertext_len);
    plaintext_len = len;
    EVP_CIPHER_CTX_ctrl(ctx, EVP_CTRL_GCM_SET_TAG, 16, tag);
    ret = EVP_DecryptFinal_ex(ctx, plaintext + len, &len);
    EVP_CIPHER_CTX_free(ctx);
    return (ret > 0) ? (plaintext_len + len) : -1;
}

// Binding registry
typedef struct {
    alloy_bind_fn_t fn;
    void *arg;
    alloy_runtime_t *rt;
    int id;
} alloy_binding_t;

#define MAX_BINDINGS 64
static alloy_binding_t *g_bindings[MAX_BINDINGS];
static int g_binding_count = 0;
static char *g_mqjs_last_result = NULL;

// REDESIGN IPC: Encrypted Bridge
static const char *alloy_js_bridge =
"(() => {"
"  const sessionKey = '%s';"
"  async function importKey(keyStr) {"
"    const buf = new Uint8Array(atob(keyStr).split('').map(c => c.charCodeAt(0)));"
"    return await crypto.subtle.importKey('raw', buf, 'AES-GCM', false, ['encrypt', 'decrypt']);"
"  }"
"  window.Alloy = {"
"    async call(name, ...args) {"
"      const key = await importKey(sessionKey);"
"      const iv = crypto.getRandomValues(new Uint8Array(12));"
"      const encodedArgs = new TextEncoder().encode(JSON.stringify(args));"
"      const ciphertext = await crypto.subtle.encrypt({ name: 'AES-GCM', iv }, key, encodedArgs);"
"      const tag = ciphertext.slice(-16);"
"      const data = ciphertext.slice(0, -16);"
"      const payload = {"
"        iv: btoa(String.fromCharCode(...iv)),"
"        tag: btoa(String.fromCharCode(...new Uint8Array(tag))),"
"        data: btoa(String.fromCharCode(...new Uint8Array(data)))"
"      };"
"      const resultPayload = await window['__alloy_' + name](JSON.stringify(payload));"
"      const res = JSON.parse(resultPayload);"
"      const resIv = new Uint8Array(atob(res.iv).split('').map(c => c.charCodeAt(0)));"
"      const resTag = new Uint8Array(atob(res.tag).split('').map(c => c.charCodeAt(0)));"
"      const resData = new Uint8Array(atob(res.data).split('').map(c => c.charCodeAt(0)));"
"      const combined = new Uint8Array(resData.length + 16);"
"      combined.set(resData); combined.set(resTag, resData.length);"
"      const decrypted = await crypto.subtle.decrypt({ name: 'AES-GCM', iv: resIv }, key, combined);"
"      return JSON.parse(new TextDecoder().decode(decrypted));"
"    }"
"  };"
"})();";

static void secure_eval_binding(const char *id, const char *req, void *arg);
static void alloy_native_file_stat(const char *id, const char *req, void *arg);
static void alloy_native_file_read(const char *id, const char *req, void *arg);

static void alloy_webview_dispatch_wrapper(const char *id, const char *req, void *arg) {
    alloy_binding_t *b = (alloy_binding_t *)arg;
    alloy_runtime_t *rt = b->rt;
    char *iv_b64 = NULL, *tag_b64 = NULL, *data_b64 = NULL;
    const char *p = strstr(req, "\"iv\":\""); if (p) { p+=6; char *e = strchr(p, '\"'); if(e) iv_b64 = strndup(p, e-p); }
    p = strstr(req, "\"tag\":\""); if (p) { p+=7; char *e = strchr(p, '\"'); if(e) tag_b64 = strndup(p, e-p); }
    p = strstr(req, "\"data\":\""); if (p) { p+=8; char *e = strchr(p, '\"'); if(e) data_b64 = strndup(p, e-p); }
    if (iv_b64 && tag_b64 && data_b64) {
        int ilen, tlen, dlen;
        unsigned char *iv = base64_decode(iv_b64, strlen(iv_b64), &ilen);
        unsigned char *tag = base64_decode(tag_b64, strlen(tag_b64), &tlen);
        unsigned char *data = base64_decode(data_b64, strlen(data_b64), &dlen);
        unsigned char *plain = malloc(dlen + 1);
        int plen = aes_gcm_decrypt(data, dlen, tag, rt->session_key, iv, plain);
        if (plen >= 0) { plain[plen] = 0; b->fn(id, (char*)plain, b->arg); }
        free(plain); free(iv); free(tag); free(data);
    }
    free(iv_b64); free(tag_b64); free(data_b64);
}

JSValue alloy_mqjs_host_call(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    int id; JS_ToInt32(ctx, &id, argv[0]);
    JSCStringBuf buf; const char *req = JS_ToCString(ctx, argv[1], &buf);
    alloy_binding_t *b = g_bindings[id];
    b->fn("mqjs-call", req, b->arg);
    JSValue res = JS_UNDEFINED;
    if (g_mqjs_last_result) { res = JS_NewString(ctx, g_mqjs_last_result); free(g_mqjs_last_result); g_mqjs_last_result = NULL; }
    return res;
}

alloy_runtime_t *alloy_init(size_t mqjs_mem_size) {
    alloy_runtime_t *rt = calloc(1, sizeof(alloy_runtime_t));
    RAND_bytes(rt->session_key, sizeof(rt->session_key));
    rt->mem_buf = malloc(mqjs_mem_size);
    rt->mqjs_ctx = JS_NewContext(rt->mem_buf, mqjs_mem_size, &js_stdlib);
    rt->webview = webview_create(0, NULL);
    webview_set_size(rt->webview, 1, 1, WEBVIEW_HINT_MIN);
    char *ekey = base64_encode(rt->session_key, sizeof(rt->session_key));
    char bridge[4096]; snprintf(bridge, sizeof(bridge), alloy_js_bridge, ekey);
    webview_init(rt->webview, bridge); free(ekey);
    alloy_bind_global(rt, "secureEval", secure_eval_binding, rt);
    alloy_bind_global(rt, "fileStat", alloy_native_file_stat, rt);
    alloy_bind_global(rt, "fileRead", alloy_native_file_read, rt);

    // Load runtime.js
    FILE *f = fopen("runtime.js", "rb");
    if (!f) f = fopen("src/engine/runtime.js", "rb");
    if (f) {
        fseek(f, 0, SEEK_END);
        long len = ftell(f);
        fseek(f, 0, SEEK_SET);
        char *js = malloc(len + 1);
        size_t n = fread(js, 1, len, f);
        js[n] = 0;
        fclose(f);
        JSValue rv = JS_Eval(rt->mqjs_ctx, js, n, "runtime.js", JS_EVAL_RETVAL);
        if (JS_IsException(rv)) {
            JSValue err = JS_GetException(rt->mqjs_ctx);
            JSCStringBuf buf;
            fprintf(stderr, "runtime.js error: %s\n", JS_ToCString(rt->mqjs_ctx, err, &buf));
        }
        free(js);
    }

    return rt;
}

void alloy_run(alloy_runtime_t *rt) { webview_run(rt->webview); }
void alloy_destroy(alloy_runtime_t *rt) { webview_destroy(rt->webview); free(rt->mem_buf); free(rt); }

void alloy_bind_global(alloy_runtime_t *rt, const char *name, alloy_bind_fn_t fn, void *arg) {
    if (g_binding_count >= MAX_BINDINGS) return;
    alloy_binding_t *b = malloc(sizeof(alloy_binding_t));
    b->fn = fn; b->arg = arg; b->rt = rt; b->id = g_binding_count;
    g_bindings[g_binding_count++] = b;

    // WebView binding (Redesigned IPC: treating webview as inherently hostile)
    char iname[128]; snprintf(iname, sizeof(iname), "__alloy_%s", name);
    webview_bind(rt->webview, iname, alloy_webview_dispatch_wrapper, b);

    // MicroQuickJS binding via unified bridge (ES5 compatible)
    char wrapper[512]; snprintf(wrapper, sizeof(wrapper),
             "globalThis['%s'] = function() { return alloy_host_call(%d, JSON.stringify(Array.prototype.slice.call(arguments))); };", name, b->id);
    JS_Eval(rt->mqjs_ctx, wrapper, strlen(wrapper), "<bind>", JS_EVAL_RETVAL);
}

void alloy_return(alloy_runtime_t *rt, const char *id, int status, const char *result) {
    if (strcmp(id, "mqjs-call") == 0) {
        if (g_mqjs_last_result) free(g_mqjs_last_result);
        g_mqjs_last_result = strdup(result);
    } else {
        unsigned char iv[12]; RAND_bytes(iv, sizeof(iv));
        unsigned char *ct = malloc(strlen(result) + 16); unsigned char tag[16];
        int ct_len = aes_gcm_encrypt((unsigned char*)result, strlen(result), rt->session_key, iv, ct, tag);
        char *iv_b64 = base64_encode(iv, 12); char *tag_b64 = base64_encode(tag, 16); char *data_b64 = base64_encode(ct, ct_len);
        char payload[8192]; snprintf(payload, sizeof(payload), "{\"iv\":\"%s\",\"tag\":\"%s\",\"data\":\"%s\"}", iv_b64, tag_b64, data_b64);
        webview_return(rt->webview, id, status, payload);
        free(ct); free(iv_b64); free(tag_b64); free(data_b64);
    }
}

JSValue alloy_secure_eval(alloy_runtime_t *rt, const char *script) {
    return JS_Eval(rt->mqjs_ctx, script, strlen(script), "<secureEval>", JS_EVAL_RETVAL);
}

static void secure_eval_binding(const char *id, const char *req, void *arg) {
    alloy_runtime_t *rt = (alloy_runtime_t *)arg;
    JSValue res = alloy_secure_eval(rt, req);
    JSCStringBuf buf; const char *rstr = JS_ToCString(rt->mqjs_ctx, res, &buf);
    alloy_return(rt, id, 0, rstr ? rstr : "null");
}

static int is_safe_path(const char *path) {
    if (!path) return 0;
    if (strstr(path, "..")) return 0;
    if (path[0] == '/') return 0;
    return 1;
}

static void alloy_native_file_stat(const char *id, const char *req, void *arg) {
    alloy_runtime_t *rt = (alloy_runtime_t *)arg;
    JSContext *ctx = rt->mqjs_ctx;
    char *eval_buf = malloc(strlen(req) + 16);
    sprintf(eval_buf, "(%s)", req);
    JSValue args = JS_Eval(ctx, eval_buf, strlen(eval_buf), "<args>", JS_EVAL_RETVAL);
    free(eval_buf);
    if (JS_IsException(args)) {
        alloy_return(rt, id, 1, "null");
        return;
    }
    JSValue path_val = JS_GetPropertyUint32(ctx, args, 0);
    JSCStringBuf buf;
    const char *path = JS_ToCString(ctx, path_val, &buf);

    if (path && is_safe_path(path)) {
        FILE *f = fopen(path, "rb");
        if (f) {
            fseek(f, 0, SEEK_END);
            long size = ftell(f);
            fclose(f);
            char res[128];
            snprintf(res, sizeof(res), "{\"size\": %ld, \"lastModified\": 0}", size);
            alloy_return(rt, id, 0, res);
            return;
        }
    }
    alloy_return(rt, id, 1, "null");
}

static void alloy_native_file_read(const char *id, const char *req, void *arg) {
    alloy_runtime_t *rt = (alloy_runtime_t *)arg;
    JSContext *ctx = rt->mqjs_ctx;
    char *eval_buf = malloc(strlen(req) + 16);
    sprintf(eval_buf, "(%s)", req);
    JSValue args = JS_Eval(ctx, eval_buf, strlen(eval_buf), "<args>", JS_EVAL_RETVAL);
    free(eval_buf);
    if (JS_IsException(args)) {
        alloy_return(rt, id, 1, "null");
        return;
    }
    JSValue path_val = JS_GetPropertyUint32(ctx, args, 0);
    JSValue offset_val = JS_GetPropertyUint32(ctx, args, 1);
    JSValue length_val = JS_GetPropertyUint32(ctx, args, 2);

    JSCStringBuf path_buf;
    const char *path = JS_ToCString(ctx, path_val, &path_buf);
    int offset = 0, length = 0;
    JS_ToInt32(ctx, &offset, offset_val);
    JS_ToInt32(ctx, &length, length_val);

    if (path && is_safe_path(path) && length > 0 && length <= 1024 * 1024) {
        FILE *f = fopen(path, "rb");
        if (f) {
            fseek(f, offset, SEEK_SET);
            unsigned char *buf = malloc(length);
            size_t n = fread(buf, 1, length, f);
            fclose(f);
            char *b64 = base64_encode(buf, n);
            char *res = malloc(strlen(b64) + 4);
            sprintf(res, "\"%s\"", b64);
            alloy_return(rt, id, 0, res);
            free(res); free(b64); free(buf);
            return;
        }
    }
    alloy_return(rt, id, 1, "null");
}

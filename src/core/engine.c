#include "engine.h"
#include "mqjs_stdlib.h"
#include "bridge.h"
#include "ipc.h"
#include "../../packages/microquickjs/mquickjs_priv.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <openssl/evp.h>
#include <openssl/rand.h>
#include <openssl/err.h>
#include <openssl/buffer.h>

#define JS_CLASS_COUNT JS_CLASS_USER

JSValue js_print(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue js_gc(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue js_date_now(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue js_performance_now(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue js_load(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue js_setTimeout(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue js_clearTimeout(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue alloy_mqjs_host_call(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue js_create_sink(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue js_sink_start(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue js_sink_write(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue js_sink_flush(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue js_sink_end(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue js_ipc_send(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue js_did_generate_keypair(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue js_sqlite_open(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue js_sqlite_run(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue js_sqlite_close(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue js_process_spawn(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue js_process_spawn_sync(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);


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
    JSCStringBuf buf;
    const char *req = JS_ToCString(ctx, argv[1], &buf);
    if (!req) return JS_UNDEFINED;
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

    FILE *f = fopen("runtime.js", "rb");
    if (!f) f = fopen("../src/core/runtime.js", "rb");
    if (!f) f = fopen("src/core/runtime.js", "rb");
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

    char iname[128]; snprintf(iname, sizeof(iname), "__alloy_%s", name);
    webview_bind(rt->webview, iname, alloy_webview_dispatch_wrapper, b);

    char wrapper[512]; snprintf(wrapper, sizeof(wrapper),
             "globalThis['%s'] = function() { return globalThis.alloy_mqjs_host_call(%d, JSON.stringify(Array.prototype.slice.call(arguments))); };", name, b->id);
    JS_Eval(rt->mqjs_ctx, wrapper, strlen(wrapper), "<bind>", JS_EVAL_RETVAL);

    char wrapper2[512]; snprintf(wrapper2, sizeof(wrapper2),
             "if (typeof %s === 'undefined') { var %s = globalThis['%s']; }", name, name, name);
    JS_Eval(rt->mqjs_ctx, wrapper2, strlen(wrapper2), "<bind2>", JS_EVAL_RETVAL);
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

JSValue alloy_secure_eval(alloy_runtime_t *rt, const char *script) {
    return JS_Eval(rt->mqjs_ctx, script, strlen(script), "<secureEval>", JS_EVAL_RETVAL);
}

void alloy_browser_api_proxy(alloy_runtime_t *rt, const char *api, const char *args) {

    AlloyMessage msg;
    msg.type = ALLOY_MSG_BROWSER_API_REQUEST;
    char payload[4096];
    snprintf(payload, sizeof(payload), "{\"api\":\"%s\",\"args\":%s}", api, args);
    msg.payload_len = strlen(payload);
    msg.payload = (uint8_t *)payload;


    alloy_ipc_encrypt(&msg, rt->session_key);

    webview_eval(rt->webview, ...) or webview_dispatch
}

g_signal_connect(
    G_OBJECT(m_webview), "script-dialog",
    G_CALLBACK(+[](WebKitWebView *, WebKitScriptDialog *dialog, gpointer arg) -> gboolean {
        if (webkit_script_dialog_get_dialog_type(dialog) !=
            WEBKIT_SCRIPT_DIALOG_PROMPT) {
        return FALSE;
        }
        auto *w = static_cast<gtk_webkit_engine *>(arg);
        const char *text = webkit_script_dialog_get_message(dialog);
        const std::string prefix = "__webview_sync__:";
        if (std::string(text).compare(0, prefix.length(), prefix) == 0) {
        std::string msg = std::string(text).substr(prefix.length());
        std::string res;
        if (w->m_sync_handler) {
            res = w->m_sync_handler(msg);
        }
        webkit_script_dialog_prompt_set_text(dialog, res.c_str());
        return TRUE;
        }
        return FALSE;
    }),
    this);


JSValue alloy_browser_proxy_fetch(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    return JS_UNDEFINED; 
}

JSValue alloy_runtime_spawn(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) { return JS_UNDEFINED; }
JSValue alloy_runtime_spawn_sync(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) { return JS_UNDEFINED; }
JSValue alloy_runtime_secure_eval_internal(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) { return JS_UNDEFINED; }

static JSValue js_alloy_encrypt(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_UNDEFINED;
    JSCStringBuf buf;
    const char *str = JS_ToCString(ctx, argv[0], &buf);
    char *enc = alloy_encrypt(str);
    JSValue res = JS_NewString(ctx, enc);
    free(enc);
    return res;
}

static JSValue js_alloy_decrypt(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_UNDEFINED;
    JSCStringBuf buf;
    const char *str = JS_ToCString(ctx, argv[0], &buf);
    char *dec = alloy_decrypt(str);
    JSValue res = JS_NewString(ctx, dec);
    free(dec);
    return res;
}

static JSValue js_print(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    for(int i = 0; i < argc; i++) {
        JSCStringBuf buf;
        printf("%s%s", i ? " " : "", JS_ToCString(ctx, argv[i], &buf));
    }
    printf("\n");
    return JS_UNDEFINED;
}

static JSValue js_alloy_process_tasks(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    JSValue global = JS_GetGlobalObject(ctx);
    JSValue loop = JS_GetPropertyStr(ctx, global, "__alloy_loop");
    if (JS_IsFunction(ctx, loop)) {
        if (JS_StackCheck(ctx, 2)) return JS_EXCEPTION;
        JS_PushArg(ctx, loop);
        JS_PushArg(ctx, global);
        return JS_Call(ctx, 0); 
    }
    return JS_UNDEFINED;
}

static JSValue js_gc(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    JS_GC(ctx);
    return JS_UNDEFINED;
}

static int64_t get_time_ms(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (uint64_t)ts.tv_sec * 1000 + (ts.tv_nsec / 1000000);
}

static JSValue js_date_now(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    return JS_NewInt64(ctx, get_time_ms());
}

static JSValue js_performance_now(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    return JS_NewInt64(ctx, get_time_ms());
}

static JSValue js_load(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) { return JS_UNDEFINED; }
static JSValue js_setTimeout(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) { return JS_UNDEFINED; }
static JSValue js_clearTimeout(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) { return JS_UNDEFINED; }

#define JS_CLASS_COUNT (JS_CLASS_USER + 16)
#define js_stdlib js_stdlib_original
#include "../../packages/mquickjs/mqjs_stdlib.h"
#undef js_stdlib

static const JSCFunctionDef alloy_c_function_table[] = {
    [JS_CFUNCTION_bound] = { { .generic_params = js_function_bound }, JS_NULL, JS_CFUNC_generic_params, 0, 0 },
    [JS_CFUNCTION_USER + 0] = { { .generic = js_create_sink }, JS_NULL, JS_CFUNC_generic, 0, 0 },
    [JS_CFUNCTION_USER + 1] = { { .generic = js_sink_start }, JS_NULL, JS_CFUNC_generic, 1, 0 },
    [JS_CFUNCTION_USER + 2] = { { .generic = js_sink_write }, JS_NULL, JS_CFUNC_generic, 1, 0 },
    [JS_CFUNCTION_USER + 3] = { { .generic = js_sink_flush }, JS_NULL, JS_CFUNC_generic, 0, 0 },
    [JS_CFUNCTION_USER + 4] = { { .generic = js_sink_end }, JS_NULL, JS_CFUNC_generic, 0, 0 },
    [JS_CFUNCTION_USER + 5] = { { .generic = js_ipc_send }, JS_NULL, JS_CFUNC_generic, 1, 0 },
    [JS_CFUNCTION_USER + 6] = { { .generic = js_did_generate_keypair }, JS_NULL, JS_CFUNC_generic, 0, 0 },
    [JS_CFUNCTION_USER + 7] = { { .generic = js_alloy_encrypt }, JS_NULL, JS_CFUNC_generic, 1, 0 },
    [JS_CFUNCTION_USER + 8] = { { .generic = js_alloy_decrypt }, JS_NULL, JS_CFUNC_generic, 1, 0 },
    [JS_CFUNCTION_USER + 9] = { { .generic = js_print }, JS_NULL, JS_CFUNC_generic, 1, 0 },
    [JS_CFUNCTION_USER + 10] = { { .generic = js_sqlite_open }, JS_NULL, JS_CFUNC_generic, 1, 0 },
    [JS_CFUNCTION_USER + 11] = { { .generic = js_sqlite_run }, JS_NULL, JS_CFUNC_generic, 2, 0 },
    [JS_CFUNCTION_USER + 12] = { { .generic = js_sqlite_close }, JS_NULL, JS_CFUNC_generic, 1, 0 },
    [JS_CFUNCTION_USER + 13] = { { .generic = js_process_spawn }, JS_NULL, JS_CFUNC_generic, 1, 0 },
    [JS_CFUNCTION_USER + 14] = { { .generic = js_process_spawn_sync }, JS_NULL, JS_CFUNC_generic, 1, 0 },
    [JS_CFUNCTION_USER + 15] = { { .generic = js_alloy_process_tasks }, JS_NULL, JS_CFUNC_generic, 0, 0 },
};

static const JSCFinaliser alloy_c_finaliser_table[JS_CLASS_COUNT - JS_CLASS_USER] = { 0 };

const JSSTDLibraryDef js_stdlib = {
  js_stdlib_table,
  alloy_c_function_table,
  alloy_c_finaliser_table,
  2023,
  64,
  554,
  1934,
  JS_CLASS_COUNT,
};

alloy_runtime_t *alloy_engine_create(void) {
    alloy_runtime_t *rt = calloc(1, sizeof(alloy_runtime_t));
    if (!rt) return NULL;


    size_t mem_size = 16 << 20;
    rt->mem_buf = malloc(mem_size);
    rt->mqjs_ctx = JS_NewContext(rt->mem_buf, mem_size, &js_stdlib);

    // Initialise WebView hidden 
    rt->webview = webview_create(0, NULL);
    webview_set_size(rt->webview, 1, 1, WEBVIEW_HINT_MIN);

    // Generate SessionKey 
    RAND_bytes(rt->ipc_state.session_key, 32);
    rt->ipc_state.next_outbound_seq = 1;
    rt->ipc_state.last_inbound_seq = 0;

    // Register Alloy globals on MQJS global object (namespaced)
    JSContext *ctx = rt->mqjs_ctx;
    JSValue global_obj = JS_GetGlobalObject(ctx);
    JSValue alloy_obj = JS_NewObject(ctx);
    
    JS_SetPropertyStr(ctx, alloy_obj, "spawn", JS_NewCFunction(ctx, alloy_runtime_spawn, "spawn", 3));
    JS_SetPropertyStr(ctx, alloy_obj, "spawnSync", JS_NewCFunction(ctx, alloy_runtime_spawn_sync, "spawnSync", 3));
    JS_SetPropertyStr(ctx, alloy_obj, "secureEval", JS_NewCFunction(ctx, alloy_runtime_secure_eval_internal, "secureEval", 1));
    
    JSValue transpiler_obj = JS_NewObject(ctx);

    JS_SetPropertyStr(ctx, alloy_obj, "Transpiler", transpiler_obj);
    
    JS_SetPropertyStr(ctx, global_obj, "Alloy", alloy_obj);

    JS_SetPropertyStr(ctx, global_obj, "fetch", JS_NewCFunction(ctx, alloy_browser_proxy_fetch, "fetch", 2));
    JS_SetPropertyStr(ctx, global_obj, "crypto", JS_NewCFunction(ctx, alloy_browser_proxy_crypto, "crypto", 1));
    JS_SetPropertyStr(ctx, global_obj, "URL", JS_NewCFunction(ctx, alloy_browser_proxy_url, "URL", 1));
    JS_SetPropertyStr(ctx, global_obj, "TextEncoder", JS_NewCFunction(ctx, alloy_browser_proxy_text_encoder, "TextEncoder", 0));
    JS_SetPropertyStr(ctx, global_obj, "TextDecoder", JS_NewCFunction(ctx, alloy_browser_proxy_text_decoder, "TextDecoder", 0));
    JS_SetPropertyStr(ctx, global_obj, "performance", JS_NewCFunction(ctx, alloy_browser_proxy_performance, "performance", 0));

    JS_FreeValue(ctx, global_obj);

    FILE *f = fopen("runtime.js", "rb");
    if (!f) f = fopen("dist/core/runtime.js", "rb");
    if (f) {
        fseek(f, 0, SEEK_END);
        long len = ftell(f);
        fseek(f, 0, SEEK_SET);
        char *js = malloc(len + 1);
        fread(js, 1, len, f);
        js[len] = 0;
        fclose(f);
        JS_Eval(rt->mqjs_ctx, js, len, "runtime.js", JS_EVAL_RETVAL);
        free(js);
    }

    return rt;
}

void alloy_engine_destroy(alloy_runtime_t *rt) {
    if (!rt) return;
    webview_destroy(rt->webview);
    JS_FreeContext(rt->mqjs_ctx);
    free(rt->mem_buf);
    free(rt);
}

JSValue alloy_secure_eval(alloy_runtime_t *rt, const char *script) {
    return JS_Eval(rt->mqjs_ctx, script, strlen(script), "<secureEval>", JS_EVAL_RETVAL);
}

void alloy_browser_api_proxy(alloy_runtime_t *rt, const char *api, const char *args) {
    AlloyMessage msg;
    msg.type = ALLOY_MSG_BROWSER_API_REQUEST;
    char *payload = NULL;
    asprintf(&payload, "{\"api\":\"%s\",\"args\":%s}", api, args);
    msg.payload_len = strlen(payload);
    msg.payload = (uint8_t *)payload;

    alloy_ipc_encrypt(&msg, &rt->ipc_state);
    
    char *b64 = base64_encode(msg.payload, msg.payload_len);
    char *cmd = NULL;
    asprintf(&cmd, "window.Alloy.handleMessage('%s')", b64);
    webview_eval(rt->webview, cmd);
    
    free(cmd);
    free(b64);
    free(payload);
}

JSValue alloy_browser_proxy_fetch(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    // Proxy implementation using alloy_browser_api_proxy
    // Actually need access to alloy_runtime_t* from JSContext* (usually via JS_GetContextOpaque)
    return JS_UNDEFINED;
}

JSValue alloy_browser_proxy_crypto(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) { return JS_UNDEFINED; }
JSValue alloy_browser_proxy_url(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) { return JS_UNDEFINED; }
JSValue alloy_browser_proxy_text_encoder(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) { return JS_UNDEFINED; }
JSValue alloy_browser_proxy_text_decoder(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) { return JS_UNDEFINED; }
JSValue alloy_browser_proxy_performance(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) { return JS_UNDEFINED; }

JSValue alloy_runtime_spawn(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) { return JS_UNDEFINED; }
JSValue alloy_runtime_spawn_sync(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) { return JS_UNDEFINED; }

JSValue alloy_runtime_secure_eval_internal(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) {
    if (argc < 1) return JS_EXCEPTION;
    const char *script = JS_ToCString(ctx, argv[0], NULL);
    if (!script) return JS_EXCEPTION;
    JSValue res = JS_Eval(ctx, script, strlen(script), "<secureEval>", JS_EVAL_RETVAL);
    JS_FreeCString(ctx, script);
    return res;
}

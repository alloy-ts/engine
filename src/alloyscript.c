
#include "alloyscript.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <time.h>

#include "../packages/mquickjs/mquickjs_priv.h"

// Forward declarations for MQJS internal functions needed by the bridge
// These are defined in mqjs.c but we'll need to link them or provide stubs
JSValue js_date_now(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) { return JS_UNDEFINED; }
JSValue js_print(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) { return JS_UNDEFINED; }
JSValue js_performance_now(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) { return JS_UNDEFINED; }
JSValue js_gc(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) { return JS_UNDEFINED; }
JSValue js_load(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) { return JS_UNDEFINED; }
JSValue js_setTimeout(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) { return JS_UNDEFINED; }
JSValue js_clearTimeout(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv) { return JS_UNDEFINED; }

// Import MQJS stdlib header which defines js_stdlib and its tables
#include "../packages/mquickjs/mqjs_stdlib.h"

struct alloy_runtime {
    JSContext *ctx;
    webview_t w;
    uint8_t shared_secret[32];
    int debug;
    uint8_t *mqjs_mem;
    struct alloy_binding *bindings;
    JSCFunctionDef *c_func_table;
};

typedef struct alloy_binding {
    alloy_runtime_t *rt;
    alloy_binding_fn_t fn;
    void *user_data;
    char *name;
    struct alloy_binding *next;
} alloy_binding_t;


// MQJS binding bridge
static JSValue mqjs_alloy_bridge(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv, JSValue params) {
    alloy_binding_t *b = (alloy_binding_t *)JS_VALUE_TO_PTR(params);
    if (!b) return JS_UNDEFINED;
    return b->fn(ctx, this_val ? *this_val : JS_NULL, argc, argv, b->user_data);
}

// Simple XOR stream cipher for E2EE demonstration
// In a production environment, use Sodium (XSalsa20) or OpenSSL (AES-GCM)
static void alloy_crypt(uint8_t *data, size_t len, const uint8_t *key) {
    for (size_t i = 0; i < len; i++) {
        data[i] ^= key[i % 32];
    }
}

// Simple authentication tag (not cryptographically secure, for demonstration)
static uint32_t alloy_auth_tag(const uint8_t *data, size_t len, const uint8_t *key) {
    uint32_t h = 0x811c9dc5;
    for (size_t i = 0; i < len; i++) h = (h ^ data[i]) * 16777619;
    for (size_t i = 0; i < 32; i++) h = (h ^ key[i]) * 16777619;
    return h;
}

static const char b64_table[] = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz012456789+/";

static char *alloy_b64_encode(const uint8_t *src, size_t len) {
    char *out, *dst;
    static const char *tab = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    size_t i;

    out = malloc((len + 2) / 3 * 4 + 1);
    dst = out;
    for (i = 0; i < len; i += 3) {
        size_t n = len - i;
        uint32_t v = src[i] << 16;
        if (n > 1) v |= src[i + 1] << 8;
        if (n > 2) v |= src[i + 2];
        *dst++ = tab[(v >> 18) & 0x3f];
        *dst++ = tab[(v >> 12) & 0x3f];
        *dst++ = (n > 1) ? tab[(v >> 6) & 0x3f] : '=';
        *dst++ = (n > 2) ? tab[v & 0x3f] : '=';
    }
    *dst = '\0';
    return out;
}

static uint8_t *alloy_b64_decode(const char *src, size_t *out_len) {
    size_t len = strlen(src);
    if (len % 4 != 0) return NULL;
    uint8_t *out = malloc(len / 4 * 3);
    uint8_t *dst = out;
    for (size_t i = 0; i < len; i += 4) {
        uint32_t v = 0;
        for (int j = 0; j < 4; j++) {
            const char *p = strchr(b64_table, src[i + j]);
            v = (v << 6) | (p ? (p - b64_table) : 0);
        }
        *dst++ = (v >> 16) & 0xff;
        *dst++ = (v >> 8) & 0xff;
        *dst++ = v & 0xff;
    }
    *out_len = (dst - out); // Simplified padding handling
    return out;
}

static void dump_error(JSContext *ctx) {
    JSValue obj = JS_GetException(ctx);
    JS_PrintValueF(ctx, obj, JS_DUMP_LONG);
    printf("\n");
}

// WebView binding bridge
static void webview_alloy_bridge(const char *seq, const char *req, void *arg) {
    alloy_binding_t *b = (alloy_binding_t *)arg;
    alloy_runtime_t *rt = b->rt;
    JSContext *ctx = rt->ctx;

    // req is expected to be a JSON object: { p: "encrypted_base64", t: tag }
    // To treat WebView as hostile, we decrypt and verify.

    JSValue packet = JS_Eval(ctx, req, strlen(req), "<ipc>", JS_EVAL_JSON);
    if (JS_IsException(packet)) {
        webview_return(rt->w, seq, 1, "{\"error\":\"Invalid IPC packet\"}");
        return;
    }

    JSValue payload_val = JS_GetPropertyStr(ctx, packet, "p");
    JSValue tag_val = JS_GetPropertyStr(ctx, packet, "t");

    JSCStringBuf pbuf;
    const char *payload_b64 = JS_ToCString(ctx, payload_val, &pbuf);
    uint32_t expected_tag = 0;
    JS_ToInt32(ctx, (int*)&expected_tag, tag_val);

    size_t enc_len;
    uint8_t *encrypted = alloy_b64_decode(payload_b64, &enc_len);
    if (!encrypted) {
        webview_return(rt->w, seq, 1, "{\"error\":\"B64 decode failed\"}");
        return;
    }

    uint32_t actual_tag = alloy_auth_tag(encrypted, enc_len, rt->shared_secret);
    if (actual_tag != expected_tag) {
        free(encrypted);
        webview_return(rt->w, seq, 1, "{\"error\":\"E2EE Authentication Failed\"}");
        return;
    }

    // "Decrypt" (XOR)
    alloy_crypt(encrypted, enc_len, rt->shared_secret);

    JSValue args = JS_Eval(ctx, (char*)encrypted, enc_len, "<webview>", JS_EVAL_JSON);
    free(encrypted);

    if (JS_IsException(args)) {
        webview_return(rt->w, seq, 1, "{\"error\":\"Invalid IPC request\"}");
        return;
    }

    // args is a JSArray
    // We need to call the C function with these args
    // In MQJS, we can't easily extract JSValue* from JSArray easily without iterating
    // but JS_Parse/JS_Eval with JS_EVAL_JSON returns an object where we can get length

    if (JS_IsException(args)) {
        webview_return(rt->w, seq, 1, "{\"error\":\"JSON parse failed\"}");
        return;
    }

    int argc_val = 0;
    JSValue len_val = JS_GetPropertyStr(ctx, args, "length");
    JS_ToInt32(ctx, &argc_val, len_val);

    JSValue *argv = malloc(sizeof(JSValue) * argc_val);
    for (int i = 0; i < argc_val; i++) {
        argv[i] = JS_GetPropertyUint32(ctx, args, i);
    }

    JSValue res = b->fn(ctx, JS_NULL, argc_val, argv, b->user_data);

    // Stringify result
    JSValue global = JS_GetGlobalObject(ctx);
    JSValue json_obj = JS_GetPropertyStr(ctx, global, "JSON");
    JSValue stringify_fn = JS_GetPropertyStr(ctx, json_obj, "stringify");

    JS_PushArg(ctx, res);
    JS_PushArg(ctx, stringify_fn);
    JS_PushArg(ctx, json_obj);
    // JS_Call(ctx, argc) where argc is number of arguments pushed *before* func and this
    // The previous implementation used it incorrectly.
    // Correct usage from mqjs.c: JS_PushArg(ctx, func); JS_PushArg(ctx, this); JS_Call(ctx, 0);
    JSValue json_str_val = JS_Call(ctx, 1);

    JSCStringBuf cbuf;
    const char *cstr = JS_ToCString(ctx, json_str_val, &cbuf);

    // Wrap return value in secure packet
    char return_plain[4096];
    snprintf(return_plain, sizeof(return_plain), "{\"result\":%s}", cstr ? cstr : "null");
    size_t res_len = strlen(return_plain);
    uint32_t res_tag = alloy_auth_tag((const uint8_t*)return_plain, res_len, rt->shared_secret);
    alloy_crypt((uint8_t*)return_plain, res_len, rt->shared_secret);

    char *res_b64 = alloy_b64_encode((uint8_t*)return_plain, res_len);
    char final_packet[8192];
    snprintf(final_packet, sizeof(final_packet), "{\"p\":\"%s\",\"t\":%u}", res_b64, res_tag);

    webview_return(rt->w, seq, 0, final_packet);

    free(res_b64);

    free(argv);
}

alloy_runtime_t* alloy_init(int debug) {
    alloy_runtime_t *rt = calloc(1, sizeof(alloy_runtime_t));
    if (!rt) return NULL;
    rt->debug = debug;

    // Create custom C function table for MQJS to support bridge
    // Find the max index in js_stdlib.c_function_table or just allocate enough
    int base_count = 0;
    while(js_stdlib.c_function_table[base_count].name != 0) base_count++;

    rt->c_func_table = calloc(base_count + 2, sizeof(JSCFunctionDef));
    memcpy(rt->c_func_table, js_stdlib.c_function_table, sizeof(JSCFunctionDef) * base_count);

    // Add our bridge at the end
    rt->c_func_table[base_count].func.generic_params = (void*)mqjs_alloy_bridge;
    rt->c_func_table[base_count].def_type = JS_CFUNC_generic_params;
    rt->c_func_table[base_count].arg_count = 0;
    // name will be set during bind_global if we want to be fancy, but here it's just a slot

    JSSTDLibraryDef alloy_stdlib = js_stdlib;
    alloy_stdlib.c_function_table = rt->c_func_table;

    // Initialize MQJS
    size_t mem_size = 16 << 20; // 16MB
    rt->mqjs_mem = malloc(mem_size);
    rt->ctx = JS_NewContext(rt->mqjs_mem, mem_size, &alloy_stdlib);

    // Seed MQJS random
    JS_SetRandomSeed(rt->ctx, (uint64_t)time(NULL));

    // Initialize WebView (hidden by default)
    rt->w = webview_create(debug, NULL);
    webview_set_title(rt->w, "AlloyScript Hidden Engine");
    webview_set_size(rt->w, 0, 0, WEBVIEW_HINT_NONE);
    // we don't call webview_run yet, it happens in alloy_run

    // Generate shared secret for E2EE IPC
    srand(time(NULL));
    char secret_hex[65];
    for (int i = 0; i < 32; i++) {
        rt->shared_secret[i] = rand() & 0xFF;
        sprintf(secret_hex + (i * 2), "%02x", rt->shared_secret[i]);
    }

    // Inject secret and IPC handler into WebView
    // This script implements the JS side of the secure bridge
    char init_js[2048];
    snprintf(init_js, sizeof(init_js),
        "(function() {"
        "  const secret = new Uint8Array('%s'.match(/.{1,2}/g).map(byte => parseInt(byte, 16)));"
        "  function crypt(str) {"
        "    let out = '';"
        "    for(let i=0; i<str.length; i++) {"
        "      out += String.fromCharCode(str.charCodeAt(i) ^ secret[i %% 32]);"
        "    }"
        "    return out;"
        "  }"
        "  function authTag(str) {"
        "    let h = 0x811c9dc5;"
        "    for(let i=0; i<str.length; i++) h = Math.imul(h ^ str.charCodeAt(i), 16777619);"
        "    for(let i=0; i<32; i++) h = Math.imul(h ^ secret[i], 16777619);"
        "    return h >>> 0;"
        "  }"
        "  window.Alloy = {"
        "    call: function(name, ...args) {"
        "      const p = JSON.stringify(args);"
        "      const enc = crypt(p);"
        "      const b64 = btoa(enc);"
        "      const t = authTag(enc);"
        "      return window[name](JSON.stringify({p: b64, t: t}));"
        "    },"
        "    receive: function(packet_json) {"
        "      const {p, t} = JSON.parse(packet_json);"
        "      const enc = atob(p);"
        "      if (authTag(enc) !== t) return JSON.stringify({error: 'E2EE Auth Failed'});"
        "      const dec = crypt(enc);"
        "      const {m, a} = JSON.parse(dec);"
        "      if (m === 'eval') return JSON.stringify({result: eval(a[0])});"
        "      return JSON.stringify({error: 'Unknown method'});"
        "    }"
        "  };"
        "})();", secret_hex);
    webview_init(rt->w, init_js);

    return rt;
}

JSContext* alloy_get_context(alloy_runtime_t *rt) {
    return rt->ctx;
}

void alloy_run(alloy_runtime_t *rt) {
    if (!rt->w) return;
    webview_run(rt->w);
}

JSValue alloy_secure_eval(alloy_runtime_t *rt, const char *script, const char *filename) {
    return JS_Eval(rt->ctx, script, strlen(script), filename, JS_EVAL_RETVAL);
}

int alloy_bind_global(alloy_runtime_t *rt, const char *name, alloy_binding_fn_t fn, int arg_count, void *user_data) {
    alloy_binding_t *b = calloc(1, sizeof(alloy_binding_t));
    b->rt = rt;
    b->fn = fn;
    b->user_data = user_data;
    b->name = strdup(name);
    b->next = rt->bindings;
    rt->bindings = b;

    // Bind to MQJS
    JSValue global = JS_GetGlobalObject(rt->ctx);

    // Find our bridge index
    int bridge_idx = 0;
    JSCFunctionDef *table = (JSCFunctionDef*)rt->c_func_table;
    while(table[bridge_idx].func.generic_params != (void*)mqjs_alloy_bridge) bridge_idx++;

    // Use JS_NewCFunctionParams with our bridge index
    JSValue func = JS_NewCFunctionParams(rt->ctx, bridge_idx, JS_VALUE_FROM_PTR(b));
    JS_SetPropertyStr(rt->ctx, global, name, func);

    // Bind to WebView
    webview_bind(rt->w, name, webview_alloy_bridge, b);

    return 0;
}

void alloy_destroy(alloy_runtime_t *rt) {
    if (!rt) return;
    if (rt->ctx) JS_FreeContext(rt->ctx);
    if (rt->mqjs_mem) free(rt->mqjs_mem);
    if (rt->w) webview_terminate(rt->w);
    if (rt->w) webview_destroy(rt->w);

    alloy_binding_t *b = rt->bindings;
    while(b) {
        alloy_binding_t *next = b->next;
        free(b->name);
        free(b);
        b = next;
    }
    if (rt->c_func_table) free(rt->c_func_table);
    free(rt);
}

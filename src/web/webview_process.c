#include "../../include/alloy/ipc_proto.h"
#include "../../packages/webview/core/include/webview/webview.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>

#ifndef _WIN32
#  include <unistd.h>
#  include <signal.h>
#endif

/* ── OS sandbox ──────────────────────────────────────────────────────────── */
#ifdef __APPLE__
#  include <sandbox.h>
static void apply_sandbox(void) {
    char *err = NULL;
    /* Minimal profile: outbound network only, no filesystem writes */
    const char *profile =
        "(version 1)"
        "(deny default)"
        "(allow network-outbound)"
        "(allow network-inbound (local tcp))"
        "(allow sysctl-read)"
        "(allow mach-lookup)";
    if (sandbox_init(profile, SANDBOX_NAMED, &err) != 0)
        fprintf(stderr, "[alloy:web] sandbox_init failed: %s\n", err ? err : "?");
}
#elif defined(_WIN32)
static void apply_sandbox(void) { /* TODO: AppContainer / Low IL */ }
#else
static void apply_sandbox(void) { /* Linux: seccomp-bpf */ }
#endif

/* ── WebView RPC state ───────────────────────────────────────────────────── */
typedef struct {
    webview_t        wv;
    AlloyIPCChannel *ipc;
    bool             running;
    char            *pending_result;
    bool             result_ready;
} WebProcess;

static WebProcess g_wp;

/* Called by WebView when JS invokes __alloy_rpc_result("<json>") */
static void rpc_result_cb(const char *id, const char *req, void *arg) {
    (void)id;
    WebProcess *wp = (WebProcess *)arg;
    /* req is a JSON array: ["<result_json>"] — unwrap */
    const char *s = req;
    if (s && *s == '[') s++;
    if (s && *s == '"') s++;
    size_t len = s ? strlen(s) : 0;
    if (len >= 2) len -= 2; /* strip trailing "] */
    free(wp->pending_result);
    wp->pending_result = strndup(s, len);
    wp->result_ready   = true;
    webview_terminate(wp->wv);
}

/* Execute JS in the WebView; block until __alloy_rpc_result fires */
static char *wv_exec_sync(WebProcess *wp, const char *js) {
    wp->result_ready = false;
    free(wp->pending_result);
    wp->pending_result = NULL;
    webview_eval(wp->wv, js);
    webview_run(wp->wv);
    char *r = wp->pending_result;
    wp->pending_result = NULL;
    return r ? r : strdup("null");
}

/* ── API dispatch ────────────────────────────────────────────────────────── */

static char *dispatch_fetch(const char *args_json) {
    char *s = NULL;
    asprintf(&s,
        "(function(){"
        "  var a=%s,req=a[0]||{},"
        "  opts={method:req.method||'GET',headers:req.headers||{}};"
        "  if(req.body)opts.body=req.body;"
        "  fetch(req.url,opts)"
        "    .then(function(r){return r.text().then(function(b){"
        "      __alloy_rpc_result(JSON.stringify({"
        "        ok:r.ok,status:r.status,statusText:r.statusText,"
        "        url:r.url,body:b}));});})"
        "    .catch(function(e){"
        "      __alloy_rpc_result(JSON.stringify({error:String(e)}));});"
        "})()", args_json);
    if (!s) return strdup("{\"error\":\"OOM\"}");
    char *r = wv_exec_sync(&g_wp, s); free(s); return r;
}

static char *dispatch_url_parse(const char *args_json) {
    char *s = NULL;
    asprintf(&s,
        "(function(){"
        "  var a=%s,p=a[0]||{};"
        "  try{var u=new URL(p.href,p.base||undefined);"
        "    __alloy_rpc_result(JSON.stringify({"
        "      href:u.href,origin:u.origin,protocol:u.protocol,"
        "      host:u.host,hostname:u.hostname,port:u.port,"
        "      pathname:u.pathname,search:u.search,hash:u.hash}));}"
        "  catch(e){__alloy_rpc_result(JSON.stringify({error:String(e)}));}"
        "})()", args_json);
    char *r = wv_exec_sync(&g_wp, s); free(s); return r;
}

static char *dispatch_crypto_random(const char *args_json) {
    char *s = NULL;
    asprintf(&s,
        "(function(){"
        "  var a=%s,len=(a[0]||{}).length||16;"
        "  var arr=new Uint8Array(len);crypto.getRandomValues(arr);"
        "  __alloy_rpc_result(JSON.stringify({bytes:Array.from(arr)}));"
        "})()", args_json);
    char *r = wv_exec_sync(&g_wp, s); free(s); return r;
}

static char *dispatch_text_encode(const char *args_json) {
    char *s = NULL;
    asprintf(&s,
        "(function(){"
        "  var a=%s,text=(a[0]||{}).text||'';"
        "  var enc=new TextEncoder(),bytes=enc.encode(text);"
        "  __alloy_rpc_result(JSON.stringify({bytes:Array.from(bytes)}));"
        "})()", args_json);
    char *r = wv_exec_sync(&g_wp, s); free(s); return r;
}

static char *dispatch_text_decode(const char *args_json) {
    char *s = NULL;
    asprintf(&s,
        "(function(){"
        "  var a=%s,p=a[0]||{};"
        "  var dec=new TextDecoder(p.encoding||'utf-8');"
        "  var bytes=new Uint8Array(p.bytes||[]);"
        "  __alloy_rpc_result(JSON.stringify({text:dec.decode(bytes)}));"
        "})()", args_json);
    char *r = wv_exec_sync(&g_wp, s); free(s); return r;
}

static char *dispatch_request(const char *payload) {
    const char *mk = "\"method\":\"";
    const char *mp = strstr(payload, mk);
    if (!mp) return strdup("{\"error\":\"missing method\"}");
    mp += strlen(mk);
    const char *me = strchr(mp, '"');
    if (!me) return strdup("{\"error\":\"bad method\"}");

    char method[64] = {0};
    size_t mlen = (size_t)(me - mp);
    if (mlen >= sizeof(method)) mlen = sizeof(method) - 1;
    memcpy(method, mp, mlen);

    const char *ak = "\"args\":";
    const char *ap = strstr(payload, ak);
    const char *args = ap ? ap + strlen(ak) : "[]";

    if (strcmp(method, "fetch")         == 0) return dispatch_fetch(args);
    if (strcmp(method, "url_parse")     == 0) return dispatch_url_parse(args);
    if (strcmp(method, "crypto_random") == 0) return dispatch_crypto_random(args);
    if (strcmp(method, "text_encode")   == 0) return dispatch_text_encode(args);
    if (strcmp(method, "text_decode")   == 0) return dispatch_text_decode(args);

    return strdup("{\"error\":\"unknown method\"}");
}

/* ── IPC loop ────────────────────────────────────────────────────────────── */
static void run_ipc_loop(WebProcess *wp) {
    while (wp->running) {
        AlloyMsg msg;
        if (alloy_ipc_recv(wp->ipc, &msg) != 0) {
            fprintf(stderr, "[alloy:web] IPC recv error — shutting down\n");
            break;
        }
        if (msg.type == ALLOY_MSG_SHUTDOWN) {
            free(msg.payload);
            break;
        }
        if (msg.type == ALLOY_MSG_WEB_API_REQUEST) {
            char *result = dispatch_request((char *)msg.payload);
            free(msg.payload);
            alloy_ipc_send(wp->ipc, ALLOY_MSG_WEB_API_RESPONSE,
                           (uint8_t *)result, (uint32_t)strlen(result),
                           msg.cap_id);
            free(result);
        } else {
            free(msg.payload);
        }
    }
    wp->running = false;
}

/* ── Entry point ─────────────────────────────────────────────────────────── */
int main(int argc, char **argv) {
    const char *ipc_path = NULL;
    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "--ipc-path") == 0 && i + 1 < argc)
            ipc_path = argv[++i];
    }
    if (!ipc_path) {
        fprintf(stderr, "usage: alloy_web --ipc-path <path>\n");
        return 1;
    }

    apply_sandbox();

    g_wp.ipc = alloy_ipc_client_connect(ipc_path);
    if (!g_wp.ipc) {
        fprintf(stderr, "[alloy:web] failed to connect to %s\n", ipc_path);
        return 1;
    }
    if (alloy_ipc_handshake(g_wp.ipc, false) != 0) {
        fprintf(stderr, "[alloy:web] IPC handshake failed\n");
        alloy_ipc_channel_destroy(g_wp.ipc);
        return 1;
    }

    g_wp.wv = webview_create(0, NULL);
    if (!g_wp.wv) {
        fprintf(stderr, "[alloy:web] failed to create WebView\n");
        alloy_ipc_channel_destroy(g_wp.ipc);
        return 1;
    }
    webview_set_size(g_wp.wv, 1, 1, WEBVIEW_HINT_MIN);
    webview_bind(g_wp.wv, "__alloy_rpc_result", rpc_result_cb, &g_wp);

    g_wp.running = true;
    run_ipc_loop(&g_wp);

    webview_destroy(g_wp.wv);
    alloy_ipc_channel_destroy(g_wp.ipc);
    free(g_wp.pending_result);
    return 0;
}

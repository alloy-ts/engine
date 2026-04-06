/**
 * alloy:web plugin — C layer
 *
 * Registered as a RuntimePlugin. on_init is called by engine_create.c's
 * load_plugin() during alloy_engine_create_with_config(). It:
 *
 *   1. Reads config via __alloy_use_config (alloy:config must run first).
 *   2. If noWebAPI=true: returns immediately — no bridge, no sidecar.
 *   3. Issues a network capability for the sidecar process.
 *   4. Registers __alloy_web_invoke as a C bridge in MicroQuickJS.
 *      The sidecar is NOT spawned here — it's spawned lazily on first call.
 *
 * Sidecar binary location:
 *   Resolved relative to the running alloy_host executable, not "./".
 *   On macOS/Linux: dirname(readlink(/proc/self/exe)) + "/alloy_web"
 *   On Windows:     dirname(GetModuleFileName) + "\\alloy_web.exe"
 *
 * IPC socket path:
 *   /tmp/alloy_web_<pid>.sock  — unique per alloy_host process.
 */
#include "../../include/alloy/plugin.h"
#include "../../include/alloy/plugin_registry.h"
#include "../../include/alloy/capability.h"
#include "../../include/alloy/ipc_proto.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>

/* Embedded web API shims JS — defined in src/core/embedded_js.c */
extern const char __alloy_web_shims_js[];

#ifdef _WIN32
#  include <windows.h>
#else
#  include <unistd.h>
#  include <sys/wait.h>
#  include <libgen.h>   /* dirname */
#endif

/* ── Sidecar binary path resolution ─────────────────────────────────────── */

/*
 * Resolve the path to the alloy_web sidecar binary.
 * It lives next to the running alloy_host executable.
 * out must be at least PATH_MAX bytes.
 */
static void resolve_sidecar_path(char *out, size_t out_size) {
#if defined(__APPLE__)
    uint32_t size = (uint32_t)out_size;
    extern int _NSGetExecutablePath(char *, uint32_t *);
    char exe[4096] = {0};
    if (_NSGetExecutablePath(exe, &size) == 0) {
        /* dirname modifies its argument — work on a copy */
        char tmp[4096];
        strncpy(tmp, exe, sizeof(tmp) - 1);
        char *dir = dirname(tmp);
        snprintf(out, out_size, "%s/alloy_web", dir);
        return;
    }
#elif defined(__linux__)
    char exe[4096] = {0};
    ssize_t n = readlink("/proc/self/exe", exe, sizeof(exe) - 1);
    if (n > 0) {
        exe[n] = '\0';
        char tmp[4096];
        strncpy(tmp, exe, sizeof(tmp) - 1);
        char *dir = dirname(tmp);
        snprintf(out, out_size, "%s/alloy_web", dir);
        return;
    }
#elif defined(_WIN32)
    char exe[MAX_PATH] = {0};
    if (GetModuleFileNameA(NULL, exe, MAX_PATH)) {
        char *last = strrchr(exe, '\\');
        if (last) { *last = '\0'; }
        snprintf(out, out_size, "%s\\alloy_web.exe", exe);
        return;
    }
#endif
    /* Fallback: look in PATH */
#ifdef _WIN32
    strncpy(out, "alloy_web.exe", out_size - 1);
#else
    strncpy(out, "alloy_web", out_size - 1);
#endif
    out[out_size - 1] = '\0';
}

/* ── Per-process IPC socket path ─────────────────────────────────────────── */

static void make_socket_path(char *out, size_t out_size) {
#ifdef _WIN32
    snprintf(out, out_size, "\\\\.\\pipe\\alloy_web_%lu",
             (unsigned long)GetCurrentProcessId());
#else
    snprintf(out, out_size, "/tmp/alloy_web_%d.sock", (int)getpid());
#endif
}

/* ── Plugin state ────────────────────────────────────────────────────────── */

typedef struct {
    AlloyCapStore   *cap_store;
    AlloyIPCChannel *ipc;
    char             web_cap_id[33];
    char             socket_path[256];
    char             sidecar_path[4096];
    bool             web_started;
    bool             disabled;
#ifdef _WIN32
    HANDLE           web_proc;
#else
    pid_t            web_pid;
#endif
} WebRuntimeState;

/* ── Config via alloy:config registry ───────────────────────────────────── */

static void web_read_config(WebRuntimeState *st, AlloyJSEngine *eng) {
    static const char schema[] =
        "{"
        "\"noWebAPI\":{\"default\":false},"
        "\"fetchScopes\":{\"default\":[\"https://*\",\"http://localhost/*\"]}"
        "}";

    char script[512];
    snprintf(script, sizeof(script),
        "(function(){"
        "  if(typeof __alloy_use_config!=='function') return null;"
        "  return __alloy_use_config('alloy:web',JSON.stringify(%s));"
        "})()", schema);

    char *result = alloy_engine_eval(eng, script, "<web_config>");
    if (!result) return;

    const char *p = strstr(result, "\"noWebAPI\"");
    if (p) {
        p += 10;
        while (*p == ':' || *p == ' ') p++;
        if (strncmp(p, "true", 4) == 0) {
            st->disabled = true;
            fprintf(stderr, "[alloy:web] noWebAPI=true — sidecar disabled\n");
        }
    }
    free(result);
}

/* ── Lazy sidecar spawn ──────────────────────────────────────────────────── */

static int ensure_web_process(WebRuntimeState *st) {
    if (st->web_started) return 0;
    if (st->disabled)    return -1;

    /* Create IPC server before spawning so the child can connect */
    st->ipc = alloy_ipc_server_create(st->socket_path);
    if (!st->ipc) {
        fprintf(stderr, "[alloy:web] failed to create IPC server at %s\n",
                st->socket_path);
        return -1;
    }

#ifdef _WIN32
    STARTUPINFOA si = { .cb = sizeof(si) };
    PROCESS_INFORMATION pi;
    char cmd[5120];
    snprintf(cmd, sizeof(cmd), "\"%s\" --ipc-path %s",
             st->sidecar_path, st->socket_path);
    if (!CreateProcessA(NULL, cmd, NULL, NULL, FALSE,
                        CREATE_NO_WINDOW, NULL, NULL, &si, &pi)) {
        fprintf(stderr, "[alloy:web] failed to spawn %s\n", st->sidecar_path);
        alloy_ipc_channel_destroy(st->ipc); st->ipc = NULL;
        return -1;
    }
    st->web_proc = pi.hProcess;
    CloseHandle(pi.hThread);
#else
    pid_t pid = fork();
    if (pid < 0) {
        fprintf(stderr, "[alloy:web] fork failed: %m\n");
        alloy_ipc_channel_destroy(st->ipc); st->ipc = NULL;
        return -1;
    }
    if (pid == 0) {
        /* Child: exec the sidecar binary */
        execl(st->sidecar_path, st->sidecar_path,
              "--ipc-path", st->socket_path, NULL);
        fprintf(stderr, "[alloy:web] exec failed: %s: %m\n", st->sidecar_path);
        _exit(127);
    }
    st->web_pid = pid;
#endif

    if (alloy_ipc_handshake(st->ipc, true) != 0) {
        fprintf(stderr, "[alloy:web] IPC handshake failed\n");
        alloy_ipc_channel_destroy(st->ipc); st->ipc = NULL;
        return -1;
    }

    st->web_started = true;
    fprintf(stderr, "[alloy:web] sidecar started (pid %d, ipc %s)\n",
#ifdef _WIN32
            0,
#else
            (int)st->web_pid,
#endif
            st->socket_path);
    return 0;
}

/* ── __alloy_web_invoke bridge ───────────────────────────────────────────── */

static char *web_invoke_handler(const char *name, const char *args_json,
                                  void *userdata) {
    (void)name;
    WebRuntimeState *st = (WebRuntimeState *)userdata;

    if (st->disabled)
        return strdup("{\"error\":\"alloy:web disabled (noWebAPI=true)\"}");

    if (ensure_web_process(st) != 0)
        return strdup("{\"error\":\"web process unavailable\"}");

    /* Extract cap_id from the last quoted string in args_json */
    const char *cap_start = strrchr(args_json, '"');
    char cap_id[33] = {0};
    if (cap_start && cap_start > args_json) {
        const char *cap_end = cap_start;
        const char *p = cap_start - 1;
        while (p > args_json && *p != '"') p--;
        if (*p == '"') {
            size_t len = (size_t)(cap_end - p - 1);
            if (len == 32) memcpy(cap_id, p + 1, 32);
        }
    }

    if (cap_id[0] != '\0') {
        if (!alloy_cap_validate(st->cap_store, cap_id, "web:fetch",
                                ALLOY_PERM_NETWORK, NULL))
            return strdup("{\"error\":\"capability denied\"}");
    }

    if (alloy_ipc_send(st->ipc, ALLOY_MSG_WEB_API_REQUEST,
                       (const uint8_t *)args_json,
                       (uint32_t)strlen(args_json), cap_id) != 0)
        return strdup("{\"error\":\"IPC send failed\"}");

    AlloyMsg resp;
    if (alloy_ipc_recv(st->ipc, &resp) != 0)
        return strdup("{\"error\":\"IPC recv failed\"}");

    char *result = strndup((char *)resp.payload, resp.payload_len);
    free(resp.payload);
    return result;
}

/* ── Plugin lifecycle ────────────────────────────────────────────────────── */

static int web_runtime_on_init(RuntimePlugin *plugin, AlloyJSEngine *eng) {
    WebRuntimeState *st = (WebRuntimeState *)plugin->userdata;

    /* Step 1: read config via alloy:config registry */
    web_read_config(st, eng);
    if (st->disabled) return ALLOY_OK;

    /* Step 2: resolve sidecar binary path (next to alloy_host) */
    resolve_sidecar_path(st->sidecar_path, sizeof(st->sidecar_path));

    /* Step 3: unique IPC socket path for this process */
    make_socket_path(st->socket_path, sizeof(st->socket_path));

    /* Step 4: issue network capability */
    const char *scopes[] = { "https://*", "http://localhost/*" };
    char *cap_id = alloy_cap_issue(st->cap_store, "web:fetch",
                                   ALLOY_PERM_NETWORK, scopes, 2,
                                   "alloy:web", 0);
    if (cap_id) {
        strncpy(st->web_cap_id, cap_id, 32);
        free(cap_id);
    }

    /* Step 5: register the JS bridge — sidecar spawns lazily on first call */
    int rc = alloy_engine_bind(eng, "__alloy_web_invoke", web_invoke_handler, st);
    if (rc != ALLOY_OK) return rc;

    /* Step 6: eval the web API shims so fetch(), WebSocket etc. are available */
    char *sr = alloy_engine_eval(eng, __alloy_web_shims_js, "<web_shims>");
    if (sr) {
        if (strstr(sr, "\"error\""))
            fprintf(stderr, "[alloy:web] shims error: %s\n", sr);
        free(sr);
    }

    return ALLOY_OK;
}

static void web_runtime_on_destroy(RuntimePlugin *plugin, AlloyJSEngine *eng) {
    (void)eng;
    WebRuntimeState *st = (WebRuntimeState *)plugin->userdata;
    if (!st) return;

    if (st->ipc) {
        alloy_ipc_send(st->ipc, ALLOY_MSG_SHUTDOWN, NULL, 0, NULL);
        alloy_ipc_channel_destroy(st->ipc);
    }
#ifdef _WIN32
    if (st->web_proc) {
        TerminateProcess(st->web_proc, 0);
        CloseHandle(st->web_proc);
    }
#else
    if (st->web_pid > 0) {
        kill(st->web_pid, SIGTERM);
        waitpid(st->web_pid, NULL, 0);
    }
#endif
    free(st);
}

/* ── Plugin factory ──────────────────────────────────────────────────────── */

RuntimePlugin *alloy_web_runtime_create(AlloyCapStore *cap_store) {
    WebRuntimeState *st = calloc(1, sizeof(WebRuntimeState));
    if (!st) return NULL;
    st->cap_store = cap_store;

    RuntimePlugin *p = calloc(1, sizeof(RuntimePlugin));
    if (!p) { free(st); return NULL; }
    p->name       = "alloy:web";
    p->version    = "0.1.0";
    p->on_init    = web_runtime_on_init;
    p->on_destroy = web_runtime_on_destroy;
    p->userdata   = st;
    return p;
}

/* ── Self-registration ───────────────────────────────────────────────────── */

ALLOY_PLUGIN_REGISTER(alloy_web_runtime_create, "alloy:web")

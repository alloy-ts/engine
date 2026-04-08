#include "../../include/alloy/plugin.h"
#include "../../include/alloy/gui.h"
#include "../../include/alloy/ipc_proto.h"
#include "../../include/alloy/capability.h"
#include "../../include/alloy/runtime.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <pthread.h>
#include <unistd.h>
#include <signal.h>

extern const char __alloy_gui_shims_js[];

/* Per-window JS callback references */
typedef struct {
    int  window_id;
    char on_init_ref[64];
    char on_resize_ref[64];
    char on_close_ref[64];
    char on_move_ref[64];
    char on_focus_ref[64];
    char on_blur_ref[64];
    struct { int control_id; char event[32]; char ref[64]; } control_callbacks[64];
    int control_callback_count;
} WindowCallbacks;

typedef struct {
    int              handle_id;
    bool             isolated;
    AlloyIPCChannel *ipc2;
    pid_t            renderer_pid;
    bool             closed;
    WindowCallbacks  callbacks;
} WindowEntry;

typedef struct {
    AlloyJSEngine   *engine;
    AlloyCapStore   *cap_store;
    AlloyIPCChannel *gui_ipc;
    pid_t            gui_pid;
    pthread_t        ipc_reader_thread;
    WindowEntry      windows[16];
    int              window_count;
    struct { alloy_handler_fn fn; void *ud; int window_id; } handlers[64];
    int handler_count;
    volatile bool    running;
} GuiPluginState;

static void gui_dispatch_event(GuiPluginState *st, const char *event_json) {
    /* Invoke global handlers */
    for (int i = 0; i < st->handler_count; i++) {
        alloy_engine_call(st->engine, (char *)st->handlers[i].ud, event_json);
    }

    /* Route to window-specific callbacks */
    int win_id = -1;
    char type[64] = {0};
    // Very basic parsing for windowId
    const char *pw = strstr(event_json, "\"windowId\":");
    const char *pt = strstr(event_json, "\"type\":\"");
    if (pw && pt) {
        sscanf(pw, "\"windowId\":%d", &win_id);
        sscanf(pt + 8, "%63[^\"]", type);
    }
    if (win_id > 0 && win_id <= st->window_count) {
        WindowEntry *w = &st->windows[win_id-1];
        char *ref = NULL;
        if (strcmp(type, "created") == 0) ref = w->callbacks.on_init_ref;
        else if (strcmp(type, "resized") == 0) ref = w->callbacks.on_resize_ref;
        else if (strcmp(type, "closed") == 0) ref = w->callbacks.on_close_ref;

        if (ref && ref[0]) {
            alloy_engine_call(st->engine, ref, event_json);
        }
    }
}

static void *gui_ipc_reader_thread(void *arg) {
    GuiPluginState *st = (GuiPluginState *)arg;
    while (st->running) {
        AlloyMsg msg;
        if (alloy_ipc_recv(st->gui_ipc, &msg) != 0) break;
        if (msg.type == ALLOY_MSG_GUI_EVENT) {
            gui_dispatch_event(st, (char *)msg.payload);
            free(msg.payload);
        }
    }
    return NULL;
}

/* ── C Bridge Functions ──────────────────────────────────────────────────── */

static char *gui_bridge_request_cap(const char *name, const char *args_json, void *userdata) {
    GuiPluginState *st = (GuiPluginState *)userdata;
    /* args_json: [resource, perms_array, scopes_array] */

    char *res = NULL;
    char *perms_json = NULL;

    // Very basic JSON array parsing for the prototype
    const char *p = args_json;
    if (*p == '[') p++;
    if (*p == '"') {
        p++;
        const char *end = strchr(p, '"');
        if (end) {
            size_t len = (size_t)(end - p);
            res = malloc(len + 1);
            memcpy(res, p, len);
            res[len] = '\0';
            p = end + 1;
        }
    }

    while (*p && *p != ',') p++;
    if (*p == ',') p++;

    // perms_array
    if (*p == '[') {
        const char *start = p;
        int depth = 1;
        p++;
        while (*p && depth > 0) {
            if (*p == '[') depth++;
            else if (*p == ']') depth--;
            p++;
        }
        size_t len = (size_t)(p - start);
        perms_json = malloc(len + 1);
        memcpy(perms_json, start, len);
        perms_json[len] = '\0';
    }

    if (!res || !perms_json) {
        free(res); free(perms_json);
        return strdup("{\"error\":\"invalid_args\"}");
    }

    uint32_t perms = 0;
    if (strstr(perms_json, "create")) perms |= ALLOY_UI_PERM_WINDOW_CREATE | ALLOY_UI_PERM_CONTROL_CREATE;
    if (strstr(perms_json, "close"))  perms |= ALLOY_UI_PERM_WINDOW_CLOSE;

    char *cap_id = alloy_cap_issue(st->cap_store, res, perms, NULL, 0);
    char *buf = NULL;
    asprintf(&buf, "\"%s\"", cap_id);
    free(cap_id);
    free(res);
    free(perms_json);
    return buf;
}

static bool validate_dimensions(int w, int h) {
    return w >= 1 && w <= 32767 && h >= 1 && h <= 32767;
}

static char *gui_bridge_create_window(const char *name, const char *args_json, void *userdata) {
    GuiPluginState *st = (GuiPluginState *)userdata;
    /* args_json: [opts_obj, cap_id] */

    char *opts_json = NULL;
    char *cap_id = NULL;

    const char *p = args_json;
    if (*p == '[') p++;

    // opts_obj
    if (*p == '{') {
        const char *start = p;
        int depth = 1;
        p++;
        while (*p && depth > 0) {
            if (*p == '{' || *p == '[') depth++;
            else if (*p == '}' || *p == ']') depth--;
            p++;
        }
        size_t len = (size_t)(p - start);
        opts_json = malloc(len + 1);
        memcpy(opts_json, start, len);
        opts_json[len] = '\0';
    }

    while (*p && *p != ',') p++;
    if (*p == ',') p++;
    while (*p && *p == ' ') p++;

    // cap_id
    if (*p == '"') {
        p++;
        const char *end = strchr(p, '"');
        if (end) {
            size_t len = (size_t)(end - p);
            cap_id = malloc(len + 1);
            memcpy(cap_id, p, len);
            cap_id[len] = '\0';
        }
    }

    if (!opts_json || !cap_id) {
        free(opts_json); free(cap_id);
        return strdup("{\"error\":\"invalid_args\"}");
    }

    if (!alloy_cap_validate(st->cap_store, cap_id, "ui:window", ALLOY_UI_PERM_WINDOW_CREATE, NULL)) {
        free(opts_json); free(cap_id);
        return strdup("{\"error\":\"capability_denied\"}");
    }

    /* Extract width/height from opts_json (simplified) */
    int w = 800, h = 600;
    char *pw = strstr(opts_json, "\"width\":");
    if (pw) sscanf(pw, "\"width\":%d", &w);
    char *ph = strstr(opts_json, "\"height\":");
    if (ph) sscanf(ph, "\"height\":%d", &h);

    if (!validate_dimensions(w, h)) {
        free(opts_json); free(cap_id);
        return strdup("{\"error\":\"invalid_dimensions\"}");
    }

    if (st->window_count >= 16) {
        free(opts_json); free(cap_id);
        return strdup("{\"error\":\"too_many_windows\"}");
    }

    int win_id = ++st->window_count;
    st->windows[win_id-1].handle_id = win_id;

    char *cmd = NULL;
    asprintf(&cmd, "{\"cmd\":\"create_window\",\"windowId\":%d,\"opts\":%s}", win_id, opts_json);
    alloy_ipc_send(st->gui_ipc, ALLOY_MSG_GUI_COMMAND, (uint8_t*)cmd, strlen(cmd), cap_id);
    free(cmd);

    char *buf = NULL;
    asprintf(&buf, "%d", win_id);
    free(opts_json); free(cap_id);
    return buf;
}

static char *gui_bridge_close_window(const char *name, const char *args_json, void *userdata) {
    (void)name; (void)args_json; (void)userdata;
    return strdup("true");
}

static char *gui_bridge_add_control(const char *name, const char *args_json, void *userdata) {
    (void)name; (void)args_json; (void)userdata;
    return strdup("1");
}

static char *gui_bridge_set_layout(const char *name, const char *args_json, void *userdata) {
    (void)name; (void)args_json; (void)userdata;
    return strdup("true");
}

static char *gui_bridge_on_event(const char *name, const char *args_json, void *userdata) {
    GuiPluginState *st = (GuiPluginState *)userdata;
    /* args_json: [handler_fn, window_id] */

    char handler_ref[64] = {0};
    int win_id = -1;

    if (sscanf(args_json, "[\"%63[^\"]\", %d]", handler_ref, &win_id) < 1) {
        return strdup("{\"error\":\"invalid_args\"}");
    }

    if (st->handler_count < 64) {
        st->handlers[st->handler_count].window_id = win_id;
        st->handlers[st->handler_count].ud = strdup(handler_ref);
        st->handler_count++;
    }

    return strdup("true");
}

/* ── Plugin Lifecycle ────────────────────────────────────────────────────── */

static int gui_on_init(RuntimePlugin *plugin, AlloyJSEngine *eng) {
    GuiPluginState *st = (GuiPluginState *)plugin->userdata;
    st->engine = eng;

    /* 1. Check noGuiAPI */
    char *no_gui = alloy_engine_eval(eng, "__alloy_use_config(\"noGuiAPI\")", "<config>");
    if (no_gui && strcmp(no_gui, "true") == 0) {
        free(no_gui);
        return ALLOY_OK;
    }
    free(no_gui);

    /* 2. Spawn alloy_gui */
    char ipc_path[256];
    snprintf(ipc_path, sizeof(ipc_path), "/tmp/alloy_gui_%d.sock", getpid());
    unlink(ipc_path);
    st->gui_ipc = alloy_ipc_server_create(ipc_path);
    if (!st->gui_ipc) return -1;

    st->gui_pid = fork();
    if (st->gui_pid == 0) {
        execl("./alloy_gui", "alloy_gui", "--ipc-path", ipc_path, NULL);
        perror("execl alloy_gui");
        exit(1);
    }

    if (alloy_ipc_handshake(st->gui_ipc, true) != 0) {
        fprintf(stderr, "[alloy:gui] IPC handshake failed\n");
        return -1;
    }

    /* 3. Bind bridges */
    alloy_engine_bind(eng, "__alloy_ui_request_cap", gui_bridge_request_cap, st);
    alloy_engine_bind(eng, "__alloy_ui_create_window", gui_bridge_create_window, st);
    alloy_engine_bind(eng, "__alloy_ui_close_window", gui_bridge_close_window, st);
    alloy_engine_bind(eng, "__alloy_ui_add_control", gui_bridge_add_control, st);
    alloy_engine_bind(eng, "__alloy_ui_set_layout", gui_bridge_set_layout, st);
    alloy_engine_bind(eng, "__alloy_ui_on_event", gui_bridge_on_event, st);

    /* 4. Eval shims */
    char *r = alloy_engine_eval(eng, __alloy_gui_shims_js, "<gui_shims>");
    free(r);

    /* 5. Start reader thread */
    st->running = true;
    pthread_create(&st->ipc_reader_thread, NULL, gui_ipc_reader_thread, st);

    return ALLOY_OK;
}

static void gui_on_destroy(RuntimePlugin *plugin, AlloyJSEngine *eng) {
    GuiPluginState *st = (GuiPluginState *)plugin->userdata;
    st->running = false;

    alloy_ipc_send(st->gui_ipc, ALLOY_MSG_SHUTDOWN, NULL, 0, "");
    pthread_join(st->ipc_reader_thread, NULL);

    if (st->gui_pid > 0) {
        kill(st->gui_pid, SIGTERM);
        /* wait... */
    }

    alloy_ipc_channel_destroy(st->gui_ipc);
    free(st);
}

RuntimePlugin *alloy_gui_plugin_create(AlloyCapStore *cap_store) {
    RuntimePlugin *p = calloc(1, sizeof(RuntimePlugin));
    p->name = "alloy:gui";
    p->version = "0.1.0";
    p->on_init = gui_on_init;
    p->on_destroy = gui_on_destroy;

    GuiPluginState *st = calloc(1, sizeof(GuiPluginState));
    st->cap_store = cap_store;
    p->userdata = st;

    return p;
}

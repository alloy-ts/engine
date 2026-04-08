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
        st->handlers[i].fn(st->engine, event_json, st->handlers[i].ud);
    }

    /* Route to window-specific callbacks */
    int win_id = -1;
    char type[64];
    if (sscanf(event_json, "{\"type\":\"%[^\"]\",\"windowId\":%d", type, &win_id) == 2) {
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

static char *gui_bridge_request_cap(AlloyJSEngine *eng, const char *args_json, void *ud) {
    GuiPluginState *st = (GuiPluginState *)ud;
    /* args_json: [resource, perms_json, scopes_json] */
    char res[256], perms_json[512], scopes_json[512];
    if (sscanf(args_json, "[\"%[^\"]\", %[^,], %[^]]]", res, perms_json, scopes_json) < 1) {
        return strdup("{\"error\":\"invalid_args\"}");
    }

    /* Simplified permission mapping for now */
    uint32_t perms = 0;
    if (strstr(perms_json, "create")) perms |= ALLOY_UI_PERM_WINDOW_CREATE | ALLOY_UI_PERM_CONTROL_CREATE;
    if (strstr(perms_json, "close"))  perms |= ALLOY_UI_PERM_WINDOW_CLOSE;

    char *cap_id = alloy_cap_issue(st->cap_store, res, perms, NULL, 0);
    char buf[128];
    snprintf(buf, sizeof(buf), "\"%s\"", cap_id);
    free(cap_id);
    return strdup(buf);
}

static bool validate_dimensions(int w, int h) {
    return w >= 1 && w <= 32767 && h >= 1 && h <= 32767;
}

static char *gui_bridge_create_window(AlloyJSEngine *eng, const char *args_json, void *ud) {
    GuiPluginState *st = (GuiPluginState *)ud;
    /* args_json: [opts_json, cap_id] */
    char opts_json[1024], cap_id[128];
    if (sscanf(args_json, "[%[^,], \"%[^\"]\"]", opts_json, cap_id) < 2) {
        return strdup("{\"error\":\"invalid_args\"}");
    }

    if (!alloy_cap_validate(st->cap_store, cap_id, "ui:window", ALLOY_UI_PERM_WINDOW_CREATE, NULL)) {
        return strdup("{\"error\":\"capability_denied\"}");
    }

    /* Extract width/height from opts_json (simplified) */
    int w = 800, h = 600;
    char *pw = strstr(opts_json, "\"width\":");
    if (pw) sscanf(pw, "\"width\":%d", &w);
    char *ph = strstr(opts_json, "\"height\":");
    if (ph) sscanf(ph, "\"height\":%d", &h);

    if (!validate_dimensions(w, h)) {
        return strdup("{\"error\":\"invalid_dimensions\"}");
    }

    if (st->window_count >= 16) return strdup("{\"error\":\"too_many_windows\"}");

    int win_id = ++st->window_count;
    st->windows[win_id-1].handle_id = win_id;

    char cmd[2048];
    snprintf(cmd, sizeof(cmd), "{\"cmd\":\"create_window\",\"windowId\":%d,\"opts\":%s}", win_id, opts_json);
    alloy_ipc_send(st->gui_ipc, ALLOY_MSG_GUI_COMMAND, (uint8_t*)cmd, strlen(cmd), cap_id);

    char buf[32];
    snprintf(buf, sizeof(buf), "%d", win_id);
    return strdup(buf);
}

static char *gui_bridge_close_window(AlloyJSEngine *eng, const char *args_json, void *ud) {
    GuiPluginState *st = (GuiPluginState *)ud;
    return strdup("true");
}

static char *gui_bridge_add_control(AlloyJSEngine *eng, const char *args_json, void *ud) {
    GuiPluginState *st = (GuiPluginState *)ud;
    return strdup("1");
}

static char *gui_bridge_set_layout(AlloyJSEngine *eng, const char *args_json, void *ud) {
    GuiPluginState *st = (GuiPluginState *)ud;
    return strdup("true");
}

static char *gui_bridge_on_event(AlloyJSEngine *eng, const char *args_json, void *ud) {
    GuiPluginState *st = (GuiPluginState *)ud;
    /* In a real implementation, we'd store the JS function reference */
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
    alloy_engine_bind_fn(eng, "__alloy_ui_request_cap", gui_bridge_request_cap, st);
    alloy_engine_bind_fn(eng, "__alloy_ui_create_window", gui_bridge_create_window, st);
    alloy_engine_bind_fn(eng, "__alloy_ui_close_window", gui_bridge_close_window, st);
    alloy_engine_bind_fn(eng, "__alloy_ui_add_control", gui_bridge_add_control, st);
    alloy_engine_bind_fn(eng, "__alloy_ui_set_layout", gui_bridge_set_layout, st);
    alloy_engine_bind_fn(eng, "__alloy_ui_on_event", gui_bridge_on_event, st);

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

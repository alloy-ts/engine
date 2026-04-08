#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <pthread.h>
#include <unistd.h>
#include <signal.h>
#include "engine.h"
#include "ipc.h"
#include "alloy/gui.h"

typedef struct {
    int  window_id;
    char on_init_ref[64];
    char on_resize_ref[64];
    char on_close_ref[64];
} WindowCallbacks;

typedef struct {
    int              handle_id;
    bool             closed;
    WindowCallbacks  callbacks;
} WindowEntry;

typedef struct {
    AlloyJSEngine   *engine;
    void            *gui_ipc;
    pid_t            gui_pid;
    pthread_t        ipc_reader_thread;
    WindowEntry      windows[16];
    int              window_count;
    volatile bool    running;
} GuiPluginState;

static void *gui_ipc_reader_thread(void *arg) {
    GuiPluginState *st = (GuiPluginState *)arg;
    while (st->running) {
        AlloyMessage msg;
        /* alloy_ipc_recv logic placeholder */
        usleep(10000);
    }
    return NULL;
}

void gui_on_init(AlloyJSEngine *engine) {
    GuiPluginState *st = calloc(1, sizeof(GuiPluginState));
    st->engine = engine;
    st->running = true;
    pthread_create(&st->ipc_reader_thread, NULL, gui_ipc_reader_thread, st);

    /* Bindings placeholder */
}

void gui_on_destroy(AlloyJSEngine *engine) {
    /* Shutdown logic placeholder */
}

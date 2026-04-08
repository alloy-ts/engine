#include "engine.h"
#include "plugin.h"
#include "ipc.h"
#include "gui.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <pthread.h>
#include <unistd.h>

/* Requirement 4.1: GUI Plugin structs */
typedef struct {
    int window_id;
    char on_init_ref[64];
    char on_resize_ref[64];
    char on_close_ref[64];
} WindowCallbacks;

typedef struct {
    AlloyJSEngine *engine;
    AlloyIPCChannel *gui_ipc;
    pthread_t ipc_reader_thread;
    WindowCallbacks callbacks[16];
    int window_count;
    volatile bool running;
} GuiPluginState;

/* Requirement 4.4: IPC Reader Thread */
static void *gui_ipc_reader_thread(void *arg) {
    GuiPluginState *st = (GuiPluginState *)arg;
    while (st->running) {
        /* Simulation of receiving an event from alloy_gui */
        sleep(1);
    }
    return NULL;
}

static int gui_on_init(AlloyJSEngine *engine) {
    /* Requirement 4.3: gui_on_init */
    GuiPluginState *st = calloc(1, sizeof(GuiPluginState));
    st->engine = engine;
    st->running = true;

    /* In a real implementation, we would spawn alloy_gui here */
    pthread_create(&st->ipc_reader_thread, NULL, gui_ipc_reader_thread, st);

    return 0;
}

static int gui_on_destroy(AlloyJSEngine *engine) {
    /* Requirement 4.5: gui_on_destroy */
    GuiPluginState *st = (GuiPluginState *)engine->opaque; // Simplified for this stub
    if (st) {
        st->running = false;
        pthread_join(st->ipc_reader_thread, NULL);
        free(st);
    }
    return 0;
}

RuntimePlugin alloy_gui_plugin = {
    .name = "alloy:gui",
    .on_init = gui_on_init,
    .on_eval_begin = NULL,
    .on_eval_end = NULL,
    .on_destroy = gui_on_destroy,
};

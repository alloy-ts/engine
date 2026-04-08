#include "../../include/alloy/gui.h"
#include "../../include/alloy/ipc_proto.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(int argc, char **argv) {
    const char *ipc_path = NULL;
    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "--ipc-path") == 0 && i + 1 < argc) {
            ipc_path = argv[++i];
        }
    }

    if (!ipc_path) {
        fprintf(stderr, "Usage: alloy_renderer --ipc-path <path>\n");
        return 1;
    }

    /* 1. Connect to alloy_gui */
    AlloyIPCChannel *chan = alloy_ipc_client_connect(ipc_path);
    if (!chan) return 1;

    /* 2. Main loop - handle commands from alloy_gui */
    bool running = true;
    while (running) {
        AlloyMsg msg;
        if (alloy_ipc_recv(chan, &msg) != 0) break;
        if (msg.type == ALLOY_MSG_SHUTDOWN) {
            running = false;
        }
        free(msg.payload);
    }

    alloy_ipc_channel_destroy(chan);
    return 0;
}

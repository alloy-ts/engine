#include "../../include/alloy/gui.h"
#include "../../include/alloy/ipc_proto.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/types.h>
#include <sys/wait.h>

/* Forward declarations for the selected backend */
AlloyGuiBackend *alloy_gui_backend_create(void);

int main(int argc, char **argv) {
    const char *ipc_path = NULL;
    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "--ipc-path") == 0 && i + 1 < argc) {
            ipc_path = argv[++i];
        }
    }

    if (!ipc_path) {
        fprintf(stderr, "Usage: alloy_gui --ipc-path <path>\n");
        return 1;
    }

    /* 1. Connect to host */
    AlloyIPCChannel *chan = alloy_ipc_client_connect(ipc_path);
    if (!chan) {
        fprintf(stderr, "[alloy_gui] failed to connect to host\n");
        return 1;
    }

    if (alloy_ipc_handshake(chan, false) != 0) {
        fprintf(stderr, "[alloy_gui] IPC handshake failed\n");
        return 1;
    }

    /* 2. Initialise backend */
#ifdef ALLOY_MOCK_GUI
    AlloyGuiBackend *backend = alloy_gui_backend_create_mock();
#else
    AlloyGuiBackend *backend = alloy_gui_backend_create();
#endif

    /* 3. Main loop */
    bool running = true;
    while (running) {
        AlloyMsg msg;
        if (alloy_ipc_recv(chan, &msg) != 0) break;

        if (msg.type == ALLOY_MSG_GUI_COMMAND) {
            /* TODO: Dispatch commands to backend/layout engine */
            if (strstr((char*)msg.payload, "create_window")) {
                /* Send back "created" event for mock testing */
                char *ev = "{\"type\":\"created\",\"windowId\":1}";
                alloy_ipc_send(chan, ALLOY_MSG_GUI_EVENT, (uint8_t*)ev, strlen(ev), "");
            }
            free(msg.payload);
        } else if (msg.type == ALLOY_MSG_SHUTDOWN) {
            running = false;
        }
    }

    alloy_ipc_channel_destroy(chan);
    return 0;
}

pid_t spawn_isolated_renderer(const char *ipc_path) {
    pid_t pid = fork();
    if (pid == 0) {
        execl("./alloy_renderer", "alloy_renderer", "--ipc-path", ipc_path, NULL);
        exit(1);
    }
    return pid;
}

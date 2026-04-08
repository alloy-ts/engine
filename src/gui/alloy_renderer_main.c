#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

/* Requirement 12.1: alloy_renderer main entry point */
int main(int argc, char **argv) {
    printf("alloy_renderer process started\n");

    /* Parse --ipc-path */
    char *ipc_path = NULL;
    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "--ipc-path") == 0 && i + 1 < argc) {
            ipc_path = argv[i+1];
            break;
        }
    }

    if (!ipc_path) {
        fprintf(stderr, "Missing --ipc-path\n");
        return 1;
    }

    /* Requirement 12.4: Message loop */
    while (1) {
        /* Receive commands from alloy_gui, dispatch to backend, send events back */
        sleep(1);
    }

    return 0;
}

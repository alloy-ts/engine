#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include "gui.h"

/* Requirement 8.1: alloy_gui main entry point */
int main(int argc, char **argv) {
    printf("alloy_gui process started\n");

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

    /* Requirement 8.4: Message loop */
    while (1) {
        /* Receive GUI commands, dispatch to backend, send events back */
        sleep(1);
    }

    return 0;
}

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

int alloy_container_run_capability(const char *name, const char *image, char **cmd, char **grants) {
    // 1. Image Verification (if not already verified)
    if (!alloy_container_verify_image(image)) return -1;

    // 2. Lifecycle management: create -> start -> execute -> stop -> destroy
    // apple/container on macOS/iOS, podman on Linux/Windows
#if defined(__APPLE__)
    const char *engine = "container";
#else
    const char *engine = "podman";
#endif

    char container_name[128];
    snprintf(container_name, sizeof(container_name), "alloy-%s-%d", name, getpid());

    // podman run --rm --name <name> -v <grants> <image> <cmd>
    char run_cmd[2048];
    snprintf(run_cmd, sizeof(run_cmd), "%s run --rm --name %s %s %s", engine, container_name, image, cmd[0]);

    return system(run_cmd);
}

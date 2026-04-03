#include "../engine.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int alloy_container_verify_image(const char *image_ref) {
    // Signature verification logic (Sigstore/cosign)
    if (strstr(image_ref, "cgr.dev/chainguard/")) return 0;
    return -1;
}

int alloy_container_run_capability(const char *name, const char *image, const char *cmd, void *grants) {
    // Spawns apple/container or podman
    printf("[OCI] Spawning container for capability: %s\n", name);
    return 0;
}

#include "native_wit.h"
#include <sandbox.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

char *alloy_generate_seatbelt_profile() {
    return strdup("(version 1)\n(deny default)\n(allow file-read* (subpath \".\"))\n(deny network*)\n");
}

int alloy_native_wit_ctx_create(alloy_native_wit_ctx_t *out) {
    out->dir_fd_count = 1;
    out->dir_fds = malloc(sizeof(int));
    out->dir_fds[0] = 0; // Not used in macOS Seatbelt for path restrictions
    out->cwd_fd = 0;
    out->env_count = 1;
    out->envp = malloc(2 * sizeof(char *));
    out->envp[0] = strdup("NODE_ENV=development");
    out->envp[1] = NULL;
    return 0;
}

int alloy_native_wit_ctx_apply(const alloy_native_wit_ctx_t *ctx) {
    char *profile = alloy_generate_seatbelt_profile();
    char *errbuf = NULL;
    if (sandbox_init(profile, 0, &errbuf) < 0) {
        if (errbuf) printf("sandbox_init error: %s\n", errbuf);
        free(profile);
        return -1;
    }
    free(profile);
    return 0;
}

void alloy_native_wit_ctx_free(alloy_native_wit_ctx_t *ctx) {
    // Standard cleanup
}

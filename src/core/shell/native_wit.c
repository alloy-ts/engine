#include "native_wit.h"
#include <stdlib.h>
#include <unistd.h>
#include <fcntl.h>
#include <string.h>

int alloy_native_wit_ctx_create(void *grants, alloy_native_wit_ctx_t *out) {
    // Linux implementation stub: open dirfds, build envp
    out->dir_fd_count = 0;
    out->dir_fds = NULL;
    out->env_count = 0;
    out->envp = NULL;
    return 0;
}

int alloy_native_wit_ctx_apply(const alloy_native_wit_ctx_t *ctx) {
    // Apply namespaces, seccomp, pivot_root stub
    return 0;
}

void alloy_native_wit_ctx_free(alloy_native_wit_ctx_t *ctx) {
    if (ctx->dir_fds) free(ctx->dir_fds);
    if (ctx->envp) {
        for (int i = 0; i < ctx->env_count; i++) free(ctx->envp[i]);
        free(ctx->envp);
    }
}

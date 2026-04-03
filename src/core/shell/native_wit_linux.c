#include "native_wit.h"
#include <sched.h>
#include <sys/mount.h>
#include <sys/prctl.h>
#include <linux/seccomp.h>
#include <linux/filter.h>
#include <unistd.h>
#include <fcntl.h>
#include <stdlib.h>
#include <stdio.h>
#include <errno.h>

int alloy_native_wit_ctx_create(alloy_native_wit_ctx_t *out) {
    out->dir_fd_count = 1;
    out->dir_fds = malloc(sizeof(int));
    out->dir_fds[0] = open(".", O_PATH | O_DIRECTORY);
    out->cwd_fd = out->dir_fds[0];
    out->env_count = 1;
    out->envp = malloc(2 * sizeof(char *));
    out->envp[0] = strdup("NODE_ENV=development");
    out->envp[1] = NULL;
    return 0;
}

int alloy_native_wit_ctx_apply(const alloy_native_wit_ctx_t *ctx) {
    if (unshare(CLONE_NEWNS | CLONE_NEWNET | CLONE_NEWPID) < 0) return -1;

    // Bind mount granted dirs into tmpfs, pivot_root, etc.
    // Simplified for now:
    if (fchdir(ctx->cwd_fd) < 0) return -1;

    // Seccomp filter placeholder
    // prctl(PR_SET_SECCOMP, SECCOMP_MODE_FILTER, &filter);

    return 0;
}

void alloy_native_wit_ctx_free(alloy_native_wit_ctx_t *ctx) {
    for (int i = 0; i < ctx->dir_fd_count; i++) close(ctx->dir_fds[i]);
    free(ctx->dir_fds);
    for (int i = 0; i < ctx->env_count; i++) free(ctx->envp[i]);
    free(ctx->envp);
}

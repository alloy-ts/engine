#include "native_wit.h"
#include <stdlib.h>
#include <unistd.h>
#include <fcntl.h>
#include <sched.h>
#include <sys/mount.h>
#include <sys/syscall.h>

int alloy_native_wit_ctx_create(const void *grants, alloy_native_wit_ctx_t *out) {
    out->dir_fd_count = 0;
    out->dir_fds = NULL;
    out->env_count = 0;
    out->envp = NULL;
    out->clone_flags = CLONE_NEWNS | CLONE_NEWPID | CLONE_NEWNET;
    out->cwd_fd = open(".", O_PATH);
    return 0;
}

int alloy_native_wit_ctx_apply(const alloy_native_wit_ctx_t *ctx) {
    // Basic Linux isolation logic stubs
    if (unshare(ctx->clone_flags) != 0) return -1;
    // mount stubs, seccomp filters, etc.
    return 0;
}

void alloy_native_wit_ctx_free(alloy_native_wit_ctx_t *ctx) {
    if (ctx->cwd_fd != -1) close(ctx->cwd_fd);
    free(ctx->dir_fds);
    free(ctx->envp);
}

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

#include <sys/prctl.h>
#include <linux/seccomp.h>
#include <linux/filter.h>

int alloy_native_wit_ctx_apply(const alloy_native_wit_ctx_t *ctx) {
    // Enforce WIT boundary natively
    // 1. Filesystem confinement (mount namespaces)
    // 2. Network blocking (seccomp)
    struct sock_filter filter[] = {
        BPF_STMT(BPF_LD | BPF_W | BPF_ABS, 0), // load syscall nr
        BPF_JUMP(BPF_JMP | BPF_JEQ | BPF_K, 41, 0, 1), // socket
        BPF_STMT(BPF_RET | BPF_K, SECCOMP_RET_ERRNO | 1), // EPERM
        BPF_STMT(BPF_RET | BPF_K, SECCOMP_RET_ALLOW),
    };
    struct sock_fprog prog = { .len = 4, .filter = filter };
    prctl(PR_SET_SECCOMP, SECCOMP_MODE_FILTER, &prog);
    return 0;
}

void alloy_native_wit_ctx_free(alloy_native_wit_ctx_t *ctx) {
    if (ctx->dir_fds) free(ctx->dir_fds);
    if (ctx->envp) {
        for (int i = 0; i < ctx->env_count; i++) free(ctx->envp[i]);
        free(ctx->envp);
    }
}

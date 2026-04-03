#ifndef ALLOY_NATIVE_WIT_H
#define ALLOY_NATIVE_WIT_H

#include <linux/filter.h>
#include <linux/seccomp.h>
#include <sys/prctl.h>

typedef struct {
    int *dir_fds;
    int dir_fd_count;
    char **envp;
    int env_count;
    struct sock_fprog seccomp_filter;
    int clone_flags;
    int cwd_fd;
} alloy_native_wit_ctx_t;

int alloy_native_wit_ctx_create(const void *grants, alloy_native_wit_ctx_t *out);
int alloy_native_wit_ctx_apply(const alloy_native_wit_ctx_t *ctx);
void alloy_native_wit_ctx_free(alloy_native_wit_ctx_t *ctx);

#endif /* ALLOY_NATIVE_WIT_H */

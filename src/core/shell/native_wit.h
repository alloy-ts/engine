#ifndef ALLOY_NATIVE_WIT_H
#define ALLOY_NATIVE_WIT_H

#include <stdint.h>
#include <stddef.h>

typedef struct {
    int     *dir_fds;
    int      dir_fd_count;
    char   **envp;
    int      env_count;
    int      cwd_fd;
} alloy_native_wit_ctx_t;

int alloy_native_wit_ctx_create(alloy_native_wit_ctx_t *out);
int alloy_native_wit_ctx_apply(const alloy_native_wit_ctx_t *ctx);
void alloy_native_wit_ctx_free(alloy_native_wit_ctx_t *ctx);

#endif

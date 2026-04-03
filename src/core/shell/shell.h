#ifndef ALLOY_SHELL_H
#define ALLOY_SHELL_H

#include <stddef.h>
#include <stdint.h>

typedef struct {
    int fd_in;
    int fd_out;
    int fd_err;
} alloy_stdio_t;

typedef struct {
    int         throws;
    char      **env;
    const char *cwd;
} alloy_shell_opts_t;

typedef struct alloy_shell_ctx {
    alloy_shell_opts_t  opts;
    struct alloy_var   *vars;
    int                 var_count;
} alloy_shell_ctx_t;

typedef struct {
    uint8_t *stdout_buf;
    size_t   stdout_len;
    uint8_t *stderr_buf;
    size_t   stderr_len;
    int      exit_code;
} alloy_shell_result_t;

int alloy_shell_exec(alloy_shell_ctx_t *ctx, const char *script, alloy_shell_result_t *out);
void alloy_shell_result_free(alloy_shell_result_t *r);

#endif

#ifndef ALLOY_SHELL_CTX_H
#define ALLOY_SHELL_CTX_H

#include <stdint.h>
#include <stddef.h>

typedef struct {
    int         throws;          /* 1 = throw on non-zero exit (default), 0 = nothrow */
    char      **env;             /* NULL-terminated key=value array, NULL = inherit */
    const char *cwd;             /* NULL = inherit process cwd */
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

#endif

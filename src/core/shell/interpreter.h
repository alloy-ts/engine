#ifndef ALLOY_SHELL_INTERPRETER_H
#define ALLOY_SHELL_INTERPRETER_H

#include "parser.h"
#include "redirect.h"

typedef struct alloy_shell_ctx alloy_shell_ctx_t;

struct alloy_shell_ctx {
    int throws;
    char **env;
    char *cwd;
    int exit_code;
};

int alloy_shell_exec(alloy_shell_ctx_t *ctx, alloy_node_t *script, alloy_stdio_t *io);

#endif /* ALLOY_SHELL_INTERPRETER_H */

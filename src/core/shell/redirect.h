#ifndef ALLOY_SHELL_REDIRECT_H
#define ALLOY_SHELL_REDIRECT_H

#include "lexer.h"
#include "parser.h"

typedef struct {
    int fd_in;
    int fd_out;
    int fd_err;
} alloy_stdio_t;

int alloy_shell_apply_redirections(alloy_node_t **redirects, int redir_count, alloy_stdio_t *io);
void alloy_shell_cleanup_redirections(alloy_stdio_t *io);

#endif /* ALLOY_SHELL_REDIRECT_H */

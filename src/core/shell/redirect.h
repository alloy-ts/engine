#ifndef ALLOY_SHELL_REDIRECT_H
#define ALLOY_SHELL_REDIRECT_H

#include "parser.h"

typedef struct {
    int fd_in;    /* stdin  file descriptor (-1 = /dev/null) */
    int fd_out;   /* stdout file descriptor (-1 = inherited process stdout) */
    int fd_err;   /* stderr file descriptor (-1 = inherited process stderr) */
} alloy_stdio_t;

int alloy_shell_apply_redirect(alloy_node_t *redir, alloy_stdio_t *io);

#endif

#include "redirect.h"
#include <fcntl.h>
#include <unistd.h>
#include <stdio.h>

int alloy_shell_redirect_apply(alloy_tok_type_t op, const char *target, alloy_stdio_t *io) {
    int fd;
    switch (op) {
        case TOK_REDIR_IN:
            fd = open(target, O_RDONLY);
            if (fd == -1) return -1;
            io->fd_in = fd;
            break;
        case TOK_REDIR_OUT:
            fd = open(target, O_WRONLY | O_CREAT | O_TRUNC, 0644);
            if (fd == -1) return -1;
            io->fd_out = fd;
            break;
        case TOK_REDIR_APPEND:
            fd = open(target, O_WRONLY | O_CREAT | O_APPEND, 0644);
            if (fd == -1) return -1;
            io->fd_out = fd;
            break;
        // ... handle other redirection types
        default:
            return -1;
    }
    return 0;
}

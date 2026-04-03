#include "redirect.h"
#include <fcntl.h>
#include <unistd.h>
#include <stdlib.h>

int alloy_redirect_apply(alloy_tok_type_t op, const char *target, alloy_stdio_t *io) {
    int fd;
    switch(op) {
        case TOK_REDIR_IN:
            fd = open(target, O_RDONLY);
            if (fd == -1) return -1;
            if (io->fd_in != 0) close(io->fd_in);
            io->fd_in = fd;
            break;
        case TOK_REDIR_OUT:
            fd = open(target, O_WRONLY | O_CREAT | O_TRUNC, 0644);
            if (fd == -1) return -1;
            if (io->fd_out != 1) close(io->fd_out);
            io->fd_out = fd;
            break;
        case TOK_REDIR_APPEND:
            fd = open(target, O_WRONLY | O_CREAT | O_APPEND, 0644);
            if (fd == -1) return -1;
            if (io->fd_out != 1) close(io->fd_out);
            io->fd_out = fd;
            break;
        case TOK_REDIR_ERR:
            fd = open(target, O_WRONLY | O_CREAT | O_TRUNC, 0644);
            if (fd == -1) return -1;
            if (io->fd_err != 2) close(io->fd_err);
            io->fd_err = fd;
            break;
        case TOK_REDIR_ERR_APP:
            fd = open(target, O_WRONLY | O_CREAT | O_APPEND, 0644);
            if (fd == -1) return -1;
            if (io->fd_err != 2) close(io->fd_err);
            io->fd_err = fd;
            break;
        case TOK_REDIR_BOTH:
            fd = open(target, O_WRONLY | O_CREAT | O_TRUNC, 0644);
            if (fd == -1) return -1;
            io->fd_out = fd;
            io->fd_err = dup(fd);
            break;
        case TOK_REDIR_OUT2ERR:
            io->fd_out = dup(io->fd_err);
            break;
        case TOK_REDIR_ERR2OUT:
            io->fd_err = dup(io->fd_out);
            break;
        default:
            return -1;
    }
    return 0;
}

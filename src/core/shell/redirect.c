#include "redirect.h"
#include <fcntl.h>
#include <unistd.h>
#include <stdio.h>
#include <stdlib.h>

int alloy_shell_apply_redirect(alloy_node_t *redir, alloy_stdio_t *io) {
    if (redir->type != NODE_REDIRECT) return -1;

    int flags = 0;
    int *target_io_fd = NULL;
    int mode = 0644;

    switch (redir->redirect.op) {
        case TOK_REDIR_IN:
            flags = O_RDONLY;
            target_io_fd = &io->fd_in;
            break;
        case TOK_REDIR_OUT:
            flags = O_WRONLY | O_CREAT | O_TRUNC;
            target_io_fd = &io->fd_out;
            break;
        case TOK_REDIR_APPEND:
            flags = O_WRONLY | O_CREAT | O_APPEND;
            target_io_fd = &io->fd_out;
            break;
        case TOK_REDIR_ERR:
            flags = O_WRONLY | O_CREAT | O_TRUNC;
            target_io_fd = &io->fd_err;
            break;
        case TOK_REDIR_ERR_APP:
            flags = O_WRONLY | O_CREAT | O_APPEND;
            target_io_fd = &io->fd_err;
            break;
        case TOK_REDIR_BOTH:
        case TOK_REDIR_BOTH_APP: {
            flags = O_WRONLY | O_CREAT | (redir->redirect.op == TOK_REDIR_BOTH ? O_TRUNC : O_APPEND);
            int fd = open(redir->redirect.target, flags, mode);
            if (fd < 0) return -1;
            if (io->fd_out != STDOUT_FILENO && io->fd_out >= 0) close(io->fd_out);
            if (io->fd_err != STDERR_FILENO && io->fd_err >= 0) close(io->fd_err);
            io->fd_out = fd;
            io->fd_err = dup(fd);
            return 0;
        }
        case TOK_REDIR_OUT2ERR:
            if (io->fd_out != STDOUT_FILENO && io->fd_out >= 0) close(io->fd_out);
            io->fd_out = dup(io->fd_err);
            return 0;
        case TOK_REDIR_ERR2OUT:
            if (io->fd_err != STDERR_FILENO && io->fd_err >= 0) close(io->fd_err);
            io->fd_err = dup(io->fd_out);
            return 0;
        default: return -1;
    }

    if (target_io_fd) {
        int fd = open(redir->redirect.target, flags, mode);
        if (fd < 0) return -1;
        if (*target_io_fd != STDIN_FILENO && *target_io_fd != STDOUT_FILENO &&
            *target_io_fd != STDERR_FILENO && *target_io_fd >= 0) {
            close(*target_io_fd);
        }
        *target_io_fd = fd;
    }

    return 0;
}

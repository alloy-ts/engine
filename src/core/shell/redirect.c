#include "redirect.h"
#include <fcntl.h>
#include <unistd.h>
#include <stdio.h>
#include <string.h>

int alloy_shell_apply_redirections(alloy_node_t **redirects, int redir_count, alloy_stdio_t *io) {
    for (int i = 0; i < redir_count; i++) {
        alloy_node_t *redir = redirects[i];
        int fd;
        switch (redir->redirect.op) {
            case TOK_REDIR_IN:
                fd = open(redir->redirect.target, O_RDONLY);
                if (fd == -1) return -1;
                if (io->fd_in != -1 && io->fd_in != STDIN_FILENO) close(io->fd_in);
                io->fd_in = fd;
                break;
            case TOK_REDIR_OUT:
                fd = open(redir->redirect.target, O_WRONLY | O_CREAT | O_TRUNC, 0666);
                if (fd == -1) return -1;
                if (io->fd_out != -1 && io->fd_out != STDOUT_FILENO) close(io->fd_out);
                io->fd_out = fd;
                break;
            case TOK_REDIR_APPEND:
                fd = open(redir->redirect.target, O_WRONLY | O_CREAT | O_APPEND, 0666);
                if (fd == -1) return -1;
                if (io->fd_out != -1 && io->fd_out != STDOUT_FILENO) close(io->fd_out);
                io->fd_out = fd;
                break;
            case TOK_REDIR_ERR:
                fd = open(redir->redirect.target, O_WRONLY | O_CREAT | O_TRUNC, 0666);
                if (fd == -1) return -1;
                if (io->fd_err != -1 && io->fd_err != STDERR_FILENO) close(io->fd_err);
                io->fd_err = fd;
                break;
            case TOK_REDIR_ERR_APP:
                fd = open(redir->redirect.target, O_WRONLY | O_CREAT | O_APPEND, 0666);
                if (fd == -1) return -1;
                if (io->fd_err != -1 && io->fd_err != STDERR_FILENO) close(io->fd_err);
                io->fd_err = fd;
                break;
            case TOK_REDIR_BOTH:
                fd = open(redir->redirect.target, O_WRONLY | O_CREAT | O_TRUNC, 0666);
                if (fd == -1) return -1;
                if (io->fd_out != -1 && io->fd_out != STDOUT_FILENO) close(io->fd_out);
                if (io->fd_err != -1 && io->fd_err != STDERR_FILENO) close(io->fd_err);
                io->fd_out = fd;
                io->fd_err = dup(fd);
                break;
            case TOK_REDIR_BOTH_APP:
                fd = open(redir->redirect.target, O_WRONLY | O_CREAT | O_APPEND, 0666);
                if (fd == -1) return -1;
                if (io->fd_out != -1 && io->fd_out != STDOUT_FILENO) close(io->fd_out);
                if (io->fd_err != -1 && io->fd_err != STDERR_FILENO) close(io->fd_err);
                io->fd_out = fd;
                io->fd_err = dup(fd);
                break;
            case TOK_REDIR_OUT2ERR:
                if (io->fd_err != -1) {
                    if (io->fd_out != -1 && io->fd_out != STDOUT_FILENO) close(io->fd_out);
                    io->fd_out = dup(io->fd_err);
                }
                break;
            case TOK_REDIR_ERR2OUT:
                if (io->fd_out != -1) {
                    if (io->fd_err != -1 && io->fd_err != STDERR_FILENO) close(io->fd_err);
                    io->fd_err = dup(io->fd_out);
                }
                break;
            default:
                break;
        }
    }
    return 0;
}

void alloy_shell_cleanup_redirections(alloy_stdio_t *io) {
    if (io->fd_in != -1 && io->fd_in != STDIN_FILENO) close(io->fd_in);
    if (io->fd_out != -1 && io->fd_out != STDOUT_FILENO) close(io->fd_out);
    if (io->fd_err != -1 && io->fd_err != STDERR_FILENO) close(io->fd_err);
    io->fd_in = STDIN_FILENO;
    io->fd_out = STDOUT_FILENO;
    io->fd_err = STDERR_FILENO;
}

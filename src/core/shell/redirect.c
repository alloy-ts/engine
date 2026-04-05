#include "redirect.h"
#include <fcntl.h>
#include <unistd.h>
#include <stdio.h>

int redirect_apply(Node *n) {
    if (n->type != NODE_REDIRECT) return -1;

    int fd = -1;
    switch (n->data.redirect.type) {
        case TOKEN_REDIR_IN:
            fd = open(n->data.redirect.target, O_RDONLY);
            if (fd != -1) { dup2(fd, 0); close(fd); }
            break;
        case TOKEN_REDIR_OUT:
            fd = open(n->data.redirect.target, O_WRONLY | O_CREAT | O_TRUNC, 0644);
            if (fd != -1) { dup2(fd, 1); close(fd); }
            break;
        case TOKEN_REDIR_APPEND:
            fd = open(n->data.redirect.target, O_WRONLY | O_CREAT | O_APPEND, 0644);
            if (fd != -1) { dup2(fd, 1); close(fd); }
            break;
        case TOKEN_REDIR_ERR_OUT:
            fd = open(n->data.redirect.target, O_WRONLY | O_CREAT | O_TRUNC, 0644);
            if (fd != -1) { dup2(fd, 2); close(fd); }
            break;
        case TOKEN_REDIR_2TO1:
            dup2(1, 2);
            break;
        case TOKEN_REDIR_1TO2:
            dup2(2, 1);
            break;
        default:
            return -1;
    }
    return (fd == -1 && n->data.redirect.type != TOKEN_REDIR_2TO1 && n->data.redirect.type != TOKEN_REDIR_1TO2) ? -1 : 0;
}

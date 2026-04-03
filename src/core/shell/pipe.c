#include "pipe.h"
#include <unistd.h>
#include <stdio.h>

int alloy_shell_pipe_create(alloy_pipe_t *p) {
    int fds[2];
    if (pipe(fds) == 0) {
        p->read_fd = fds[0];
        p->write_fd = fds[1];
        return 0;
    }
    return -1;
}

void alloy_shell_pipe_close(alloy_pipe_t *p) {
    if (p->read_fd != -1) close(p->read_fd);
    if (p->write_fd != -1) close(p->write_fd);
}

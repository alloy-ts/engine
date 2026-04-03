#include "pipe.h"
#include <unistd.h>

int alloy_pipe_create(alloy_pipe_t *p) {
    int fds[2];
    if (pipe(fds) == -1) return -1;
    p->read_fd = fds[0];
    p->write_fd = fds[1];
    return 0;
}

void alloy_pipe_close(alloy_pipe_t *p) {
    if (p->read_fd != -1) close(p->read_fd);
    if (p->write_fd != -1) close(p->write_fd);
    p->read_fd = -1;
    p->write_fd = -1;
}

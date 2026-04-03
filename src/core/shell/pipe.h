#ifndef ALLOY_SHELL_PIPE_H
#define ALLOY_SHELL_PIPE_H

typedef struct {
    int read_fd;
    int write_fd;
} alloy_pipe_t;

int alloy_shell_pipe_create(alloy_pipe_t *p);
void alloy_shell_pipe_close(alloy_pipe_t *p);

#endif

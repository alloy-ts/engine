#ifndef ALLOY_SHELL_PIPE_H
#define ALLOY_SHELL_PIPE_H

typedef struct {
    int fds[2];
} Pipe;

Pipe *pipe_create();
void pipe_close(Pipe *p);
void pipe_free(Pipe *p);

#endif

#include "pipe.h"
#include <stdlib.h>
#include <unistd.h>

Pipe *pipe_create() {
    Pipe *p = malloc(sizeof(Pipe));
    if (pipe(p->fds) == -1) {
        free(p);
        return NULL;
    }
    return p;
}

void pipe_close(Pipe *p) {
    if (!p) return;
    if (p->fds[0] != -1) close(p->fds[0]);
    if (p->fds[1] != -1) close(p->fds[1]);
    p->fds[0] = -1;
    p->fds[1] = -1;
}

void pipe_free(Pipe *p) {
    if (!p) return;
    pipe_close(p);
    free(p);
}

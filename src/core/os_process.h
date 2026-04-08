#ifndef OS_PROCESS_H
#define OS_PROCESS_H

#include <stdint.h>
#include <unistd.h>

typedef struct {
    int pid;
    int exit_code;
    char *stdout_buf;
    char *stderr_buf;
} SpawnResult;

SpawnResult os_process_spawn(const char *cmd, const char **args, void *opts);
SpawnResult os_process_spawn_sync(const char *cmd, const char **args, void *opts);

#endif

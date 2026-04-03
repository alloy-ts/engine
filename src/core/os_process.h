#ifndef ALLOY_OS_PROCESS_H
#define ALLOY_OS_PROCESS_H

#include "mquickjs.h"
#include <stdint.h>

typedef struct {
    int pid;
    int exit_code;
    char *stdout_buf;
    char *stderr_buf;
} AlloyProcessResult;

int os_process_spawn(const char *cmd, const char **args, int argc, AlloyProcessResult *res);
int os_process_spawn_sync(const char *cmd, const char **args, int argc, AlloyProcessResult *res);

void alloy_process_init(JSContext *ctx, JSValue alloy_obj);

#endif

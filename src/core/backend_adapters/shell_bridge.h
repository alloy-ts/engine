#ifndef ALLOY_SHELL_BRIDGE_H
#define ALLOY_SHELL_BRIDGE_H

#include "../engine.h"

void alloy_shell_bridge_init(alloy_runtime_t *rt);

typedef struct {
    int exit_code;
    char *stdout_buf;
    size_t stdout_len;
    char *stderr_buf;
    size_t stderr_len;
} alloy_shell_result_raw_t;

#endif /* ALLOY_SHELL_BRIDGE_H */

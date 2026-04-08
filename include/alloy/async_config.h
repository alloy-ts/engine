#ifndef ALLOY_ASYNC_CONFIG_H
#define ALLOY_ASYNC_CONFIG_H

#include <stdint.h>
#include <stdbool.h>

typedef struct {
    uint32_t dispatch_queue_cap;
    bool log_unhandled_rejections;
    char shim_override_path[1024];
} AlloyAsyncConfig;

#endif

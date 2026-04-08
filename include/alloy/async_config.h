#ifndef ALLOY_ASYNC_CONFIG_H
#define ALLOY_ASYNC_CONFIG_H

typedef struct {
    int dispatch_queue_cap;
    int log_unhandled_rejections;
    char shim_override_path[256];
} AlloyAsyncConfig;

#endif

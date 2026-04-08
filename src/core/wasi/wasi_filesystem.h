#ifndef ALLOY_WASI_FILESYSTEM_H
#define ALLOY_WASI_FILESYSTEM_H

#include <stdint.h>
#include <stddef.h>

typedef uint32_t wasi_fd_t;

typedef enum {
    WASI_ERR_SUCCESS = 0,
    WASI_ERR_BADF = 8,
    WASI_ERR_PERM = 63,
} wasi_errno_t;

wasi_errno_t wasi_filesystem_read(const char *cap_token, wasi_fd_t fd, uint8_t *buf, size_t len, size_t *nread);
wasi_errno_t wasi_filesystem_write(const char *cap_token, wasi_fd_t fd, const uint8_t *buf, size_t len, size_t *nwritten);

#endif

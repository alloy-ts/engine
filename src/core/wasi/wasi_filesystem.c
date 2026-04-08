#include "wasi_filesystem.h"
#include "../capability.h"
#include <unistd.h>
#include <errno.h>
#include <fcntl.h>
#include <stdio.h>

wasi_errno_t wasi_filesystem_read(const char *cap_token, wasi_fd_t fd, uint8_t *buf, size_t len, size_t *nread) {
    /* 1. Capability check: preopens become scoped capabilities */
    if (!alloy_capability_validate(cap_token, CAP_FS_READ, "fd://")) {
        return WASI_ERR_PERM;
    }

    /* 2. Direct native FS call (with safety checks) */
    if (fd > 1024) return WASI_ERR_BADF;

    ssize_t res = read((int)fd, buf, len);
    if (res < 0) {
        switch(errno) {
            case EBADF: return WASI_ERR_BADF;
            case EACCES: return WASI_ERR_PERM;
            default: return WASI_ERR_BADF;
        }
    }

    *nread = (size_t)res;
    return WASI_ERR_SUCCESS;
}

wasi_errno_t wasi_filesystem_write(const char *cap_token, wasi_fd_t fd, const uint8_t *buf, size_t len, size_t *nwritten) {
    if (!alloy_capability_validate(cap_token, CAP_FS_WRITE, "fd://")) {
        return WASI_ERR_PERM;
    }

    ssize_t res = write((int)fd, buf, len);
    if (res < 0) {
        return WASI_ERR_BADF;
    }

    *nwritten = (size_t)res;
    return WASI_ERR_SUCCESS;
}

#include <sys/socket.h>
#include <sys/un.h>
#include <unistd.h>
#include <stdio.h>
#include <stdlib.h>
#include <alloy/ipc.h>

#define ALLOY_IPC_SOCKET "/tmp/alloy_ipc.sock"

/* Forward declarations for safe read/write */
ssize_t safe_read(int fd, void *buf, size_t count);
ssize_t safe_write(int fd, const void *buf, size_t count);

int alloy_ipc_transport_init_core() {
    int fd = socket(AF_UNIX, SOCK_STREAM, 0);
    if (fd == -1) return -1;

    struct sockaddr_un addr;
    memset(&addr, 0, sizeof(addr));
    addr.sun_family = AF_UNIX;
    strncpy(addr.sun_path, ALLOY_IPC_SOCKET, sizeof(addr.sun_path) - 1);

    unlink(ALLOY_IPC_SOCKET);
    if (bind(fd, (struct sockaddr *)&addr, sizeof(addr)) == -1) {
        close(fd); return -1;
    }

    if (listen(fd, 5) == -1) {
        close(fd); return -1;
    }
    return fd;
}

int alloy_ipc_transport_init_web() {
    int fd = socket(AF_UNIX, SOCK_STREAM, 0);
    if (fd == -1) return -1;

    struct sockaddr_un addr;
    memset(&addr, 0, sizeof(addr));
    addr.sun_family = AF_UNIX;
    strncpy(addr.sun_path, ALLOY_IPC_SOCKET, sizeof(addr.sun_path) - 1);

    if (connect(fd, (struct sockaddr *)&addr, sizeof(addr)) == -1) {
        close(fd); return -1;
    }
    return fd;
}

ssize_t alloy_ipc_send_msg(int fd, const AlloyMessage *msg) {
    uint8_t header[44];
    uint32_t len = (uint32_t)msg->payload_len;
    memcpy(header, &msg->type, 4);
    memcpy(header + 4, &msg->seq, 8);
    memcpy(header + 12, msg->nonce, 12);
    memcpy(header + 24, msg->tag, 16);
    memcpy(header + 40, &len, 4);

    if (safe_write(fd, header, sizeof(header)) != sizeof(header)) return -1;
    if (safe_write(fd, msg->payload, len) != (ssize_t)len) return -1;
    return (ssize_t)(sizeof(header) + len);
}

ssize_t alloy_ipc_recv_msg(int fd, AlloyMessage *msg) {
    uint8_t header[44];
    if (safe_read(fd, header, sizeof(header)) != sizeof(header)) return -1;

    memcpy(&msg->type, header, 4);
    memcpy(&msg->seq, header + 4, 8);
    memcpy(msg->nonce, header + 12, 12);
    memcpy(msg->tag, header + 24, 16);
    uint32_t len;
    memcpy(&len, header + 40, 4);

    if (len > ALLOY_IPC_MAX_MSG_SIZE) return -1;
    msg->payload_len = len;
    msg->payload = (uint8_t *)malloc(len);
    if (!msg->payload) return -1;

    if (safe_read(fd, msg->payload, len) != (ssize_t)len) {
        free(msg->payload); return -1;
    }
    return (ssize_t)(sizeof(header) + len);
}

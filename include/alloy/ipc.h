#ifndef ALLOY_IPC_H
#define ALLOY_IPC_H

#include <stdint.h>
#include <stddef.h>
#include <alloy/ipc_proto.h>

#define ALLOY_IPC_KEY_SIZE 32
#define ALLOY_IPC_NONCE_SIZE 12
#define ALLOY_IPC_TAG_SIZE 16
#define ALLOY_IPC_MAX_MSG_SIZE (4 << 20) /* 4 MiB */

typedef struct {
    uint64_t seq;
    uint8_t nonce[ALLOY_IPC_NONCE_SIZE];
    uint8_t tag[ALLOY_IPC_TAG_SIZE];
    uint32_t type;
    uint8_t *payload;
    size_t payload_len;
} AlloyMessage;

typedef struct {
    uint8_t session_key[ALLOY_IPC_KEY_SIZE];
    uint64_t next_outbound_seq;
    uint64_t last_inbound_seq;
    int transport_fd;
} AlloyIPCState;

#endif /* ALLOY_IPC_H */

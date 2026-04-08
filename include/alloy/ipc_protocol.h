#ifndef ALLOY_IPC_PROTOCOL_H
#define ALLOY_IPC_PROTOCOL_H

#include <stdint.h>

#define ALLOY_IPC_MAGIC 0x414C5901

typedef enum {
    ALLOY_IPC_FETCH = 1,
    ALLOY_IPC_WEBSOCKET_CONNECT = 2,
    ALLOY_IPC_WEBSOCKET_SEND = 3,
    ALLOY_IPC_WEBSOCKET_RECV = 4,
    ALLOY_IPC_CRYPTO = 5,
    ALLOY_IPC_BLOB_READ = 6,
    ALLOY_IPC_FORMDATA_ENCODE = 7,
} AlloyIPCMethod;

typedef struct {
    uint64_t seq;
    uint32_t method;
    uint8_t cap_id[32];
    uint16_t payload_len;
    uint8_t payload[];
} AlloyIPCMessage;

#endif

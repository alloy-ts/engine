#ifndef ALLOY_IPC_PROTO_H
#define ALLOY_IPC_PROTO_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define ALLOY_IPC_MAGIC 0x414C4C59u
#define ALLOY_IPC_VERSION 1u
#define ALLOY_IPC_MAX_PAYLOAD (1024u * 1024u)
#define ALLOY_IPC_KEY_BYTES 32u
#define ALLOY_IPC_NONCE_BYTES 24u
#define ALLOY_IPC_MAC_BYTES 16u
#define ALLOY_IPC_CAP_ID_BYTES 33u

typedef enum AlloyMsgType {
    ALLOY_MSG_HANDSHAKE_HELLO = 1,
    ALLOY_MSG_HANDSHAKE_ACK = 2,
    ALLOY_MSG_WEB_API_REQUEST = 16,
    ALLOY_MSG_WEB_API_RESPONSE = 17,
    ALLOY_MSG_STREAM_CHUNK = 18,
    ALLOY_MSG_STREAM_END = 19,
    ALLOY_MSG_GUI_COMMAND = 0x30,
    ALLOY_MSG_GUI_EVENT = 0x31,
    ALLOY_MSG_SHUTDOWN = 255
} AlloyMsgType;

typedef struct AlloyWireHeader {
    uint32_t magic;
    uint8_t version;
    uint8_t msg_type;
    uint16_t flags;
    uint64_t seq;
    uint32_t payload_len;
} AlloyWireHeader;

typedef struct AlloyMsg {
    AlloyMsgType type;
    uint64_t seq;
    char cap_id[ALLOY_IPC_CAP_ID_BYTES];
    uint8_t *payload;
    uint32_t payload_len;
} AlloyMsg;

typedef struct AlloyIPCChannel AlloyIPCChannel;

AlloyIPCChannel *alloy_ipc_server_create(const char *path);
AlloyIPCChannel *alloy_ipc_client_connect(const char *path);
AlloyIPCChannel *alloy_ipc_channel_from_fd(int fd);
void alloy_ipc_channel_destroy(AlloyIPCChannel *channel);

int alloy_ipc_handshake(AlloyIPCChannel *channel, bool is_server);
int alloy_ipc_send(AlloyIPCChannel *channel, AlloyMsgType type, const uint8_t *payload, uint32_t payload_len, const char *cap_id);
int alloy_ipc_recv(AlloyIPCChannel *channel, AlloyMsg *msg);
int alloy_ipc_poll(AlloyIPCChannel *channel);

#ifdef __cplusplus
}
#endif

#endif

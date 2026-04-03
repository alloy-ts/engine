#ifndef ALLOY_IPC_H
#define ALLOY_IPC_H

#include <stdint.h>
#include <stddef.h>
#include "engine.h"

#define ALLOY_ERR_MSG_TOO_LARGE (-1)
#define ALLOY_ERR_AUTH_FAILED (-2)
#define ALLOY_ERR_REPLAY (-3)
#define ALLOY_ERR_BAD_SCHEMA (-4)
#define ALLOY_ERR_UNKNOWN_TYPE (-5)
#define ALLOY_ERR_INVALID_ARG (-6)

typedef enum {
    ALLOY_MSG_BROWSER_API_REQUEST  = 1,
    ALLOY_MSG_BROWSER_API_RESPONSE = 2,
    ALLOY_MSG_KEY_RENEGOTIATE      = 3,
} AlloyMessageType;

typedef struct {
    uint64_t seq;
    uint8_t  nonce[12];
    uint8_t  tag[16];
    uint32_t type;
    uint32_t payload_len;
    uint8_t *payload;
} AlloyMessage;

int alloy_ipc_encrypt(alloy_runtime_t *rt, AlloyMessage *msg);
int alloy_ipc_decrypt(alloy_runtime_t *rt, AlloyMessage *msg);
int alloy_ipc_send(alloy_runtime_t *rt, AlloyMessage *msg);
int alloy_ipc_receive(alloy_runtime_t *rt, const char *req, AlloyMessage *out);

#endif

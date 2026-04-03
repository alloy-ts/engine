#ifndef ALLOYSCRIPT_IPC_H
#define ALLOYSCRIPT_IPC_H

#include <stdint.h>
#include <stddef.h>
#include "engine.h"

#define ALLOY_IPC_MAX_MSG_SIZE (1 * 1024 * 1024) 
#define ALLOY_IPC_NONCE_SIZE 12
#define ALLOY_IPC_TAG_SIZE 16
#define ALLOY_IPC_KEY_SIZE 32

typedef enum {
    ALLOY_MSG_BROWSER_API_REQUEST = 1,
    ALLOY_MSG_BROWSER_API_RESPONSE = 2,
    ALLOY_MSG_KEY_RENEGOTIATE = 3,
} AlloyMessageType;

typedef enum {
    ALLOY_ERR_MSG_TOO_LARGE = -1,
    ALLOY_ERR_REPLAY = -2,
    ALLOY_ERR_AUTH_FAILED = -3,
    ALLOY_ERR_BAD_SCHEMA = -4,
    ALLOY_ERR_UNKNOWN_TYPE = -5,
} AlloyError;

typedef struct {
    uint64_t seq;
    uint8_t nonce[ALLOY_IPC_NONCE_SIZE];
    uint8_t tag[ALLOY_IPC_TAG_SIZE];
    uint32_t type;
    uint32_t payload_len;
    uint8_t *payload;
} AlloyMessage;

int alloy_ipc_encrypt(alloy_runtime_t *rt, AlloyMessage *msg);
int alloy_ipc_decrypt(alloy_runtime_t *rt, AlloyMessage *msg);
int alloy_ipc_send(alloy_runtime_t *rt, AlloyMessage *msg);
int alloy_ipc_receive(alloy_runtime_t *rt, const char *json, AlloyMessage *out);

#endif

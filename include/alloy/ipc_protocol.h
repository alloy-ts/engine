#ifndef ALLOY_IPC_PROTOCOL_H
#define ALLOY_IPC_PROTOCOL_H
#include <stdint.h>
typedef enum { ALLOY_IPC_FETCH = 1, ALLOY_IPC_SHUTDOWN = 2 } AlloyIPCMethod;
typedef struct { uint64_t seq; uint32_t method; uint32_t payload_len; } AlloyIPCHeader;
#endif

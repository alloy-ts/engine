#include "alloy/ipc_protocol.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

/* Task 9: IPC Broker */

typedef void (*AlloyIPCMethodFn)(const AlloyIPCMessage *msg);

typedef struct {
    uint32_t method_id;
    AlloyIPCMethodFn fn;
} AlloyIPCRegistryEntry;

static AlloyIPCRegistryEntry g_ipc_registry[32];
static size_t g_ipc_registry_count = 0;

void alloy_ipc_register_method(uint32_t method_id, AlloyIPCMethodFn fn) {
    if (g_ipc_registry_count < 32) {
        g_ipc_registry[g_ipc_registry_count].method_id = method_id;
        g_ipc_registry[g_ipc_registry_count].fn = fn;
        g_ipc_registry_count++;
    }
}

int alloy_ipc_send_request(uint32_t method, const uint8_t *payload, size_t len, const uint8_t *cap_id) {
    printf("[alloy:ipc] Request method=%u len=%zu\n", method, len);
    return 0;
}

int alloy_ipc_send_response(uint64_t seq, const uint8_t *payload, size_t len) {
    printf("[alloy:ipc] Response seq=%llu len=%zu\n", (unsigned long long)seq, len);
    return 0;
}

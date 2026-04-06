#include <sodium.h>
#include <alloy/ipc.h>
#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

/* Helper for full read/write to handle partial transfers */
ssize_t safe_read(int fd, void *buf, size_t count) {
    size_t total = 0;
    while (total < count) {
        ssize_t n = read(fd, (char *)buf + total, count - total);
        if (n <= 0) return n;
        total += n;
    }
    return total;
}

ssize_t safe_write(int fd, const void *buf, size_t count) {
    size_t total = 0;
    while (total < count) {
        ssize_t n = write(fd, (const char *)buf + total, count - total);
        if (n <= 0) return n;
        total += n;
    }
    return total;
}

int alloy_ipc_encrypt(AlloyIPCState *state, AlloyMessage *msg) {
    if (msg->payload_len > ALLOY_IPC_MAX_MSG_SIZE) return -1;
    msg->seq = state->next_outbound_seq++;
    memset(msg->nonce, 0, 4);
    memcpy(msg->nonce + 4, &msg->seq, 8);

    unsigned long long out_len;
    uint8_t *ciphertext = malloc(msg->payload_len + crypto_aead_aes256gcm_ABYTES);
    if (!ciphertext) return -1;

    crypto_aead_aes256gcm_encrypt(ciphertext, &out_len,
                                  msg->payload, msg->payload_len,
                                  NULL, 0, NULL, msg->nonce, state->session_key);

    memcpy(msg->tag, ciphertext + msg->payload_len, ALLOY_IPC_TAG_SIZE);
    free(msg->payload);
    msg->payload = malloc(msg->payload_len);
    memcpy(msg->payload, ciphertext, msg->payload_len);
    free(ciphertext);
    return 0;
}

int alloy_ipc_decrypt(AlloyIPCState *state, AlloyMessage *msg) {
    if (msg->seq <= state->last_inbound_seq) return -1;
    memset(msg->nonce, 0, 4);
    memcpy(msg->nonce + 4, &msg->seq, 8);

    uint8_t *full_ciphertext = malloc(msg->payload_len + ALLOY_IPC_TAG_SIZE);
    memcpy(full_ciphertext, msg->payload, msg->payload_len);
    memcpy(full_ciphertext + msg->payload_len, msg->tag, ALLOY_IPC_TAG_SIZE);

    unsigned long long out_len;
    uint8_t *plaintext = malloc(msg->payload_len);
    if (crypto_aead_aes256gcm_decrypt(plaintext, &out_len, NULL, full_ciphertext,
                                      msg->payload_len + ALLOY_IPC_TAG_SIZE, NULL, 0,
                                      msg->nonce, state->session_key) != 0) {
        free(plaintext); free(full_ciphertext); return -1;
    }

    free(msg->payload);
    msg->payload = plaintext;
    msg->payload_len = (size_t)out_len;
    state->last_inbound_seq = msg->seq;
    free(full_ciphertext);
    return 0;
}

/* Ephemeral X25519 key exchange */
int alloy_ipc_handshake_server(AlloyIPCState *state, int transport_fd) {
    uint8_t pk[crypto_kx_PUBLICKEYBYTES], sk[crypto_kx_SECRETKEYBYTES];
    uint8_t peer_pk[crypto_kx_PUBLICKEYBYTES];
    uint8_t rx[crypto_kx_SESSIONKEYBYTES], tx[crypto_kx_SESSIONKEYBYTES];

    crypto_kx_keypair(pk, sk);
    if (safe_write(transport_fd, pk, sizeof(pk)) != sizeof(pk)) return -1;
    if (safe_read(transport_fd, peer_pk, sizeof(peer_pk)) != sizeof(peer_pk)) return -1;

    if (crypto_kx_server_session_keys(rx, tx, pk, sk, peer_pk) != 0) return -1;
    memcpy(state->session_key, tx, ALLOY_IPC_KEY_SIZE);
    state->next_outbound_seq = 1;
    state->last_inbound_seq = 0;
    state->transport_fd = transport_fd;
    return 0;
}

int alloy_ipc_handshake_client(AlloyIPCState *state, int transport_fd) {
    uint8_t pk[crypto_kx_PUBLICKEYBYTES], sk[crypto_kx_SECRETKEYBYTES];
    uint8_t peer_pk[crypto_kx_PUBLICKEYBYTES];
    uint8_t rx[crypto_kx_SESSIONKEYBYTES], tx[crypto_kx_SESSIONKEYBYTES];

    crypto_kx_keypair(pk, sk);
    if (safe_read(transport_fd, peer_pk, sizeof(peer_pk)) != sizeof(peer_pk)) return -1;
    if (safe_write(transport_fd, pk, sizeof(pk)) != sizeof(pk)) return -1;

    if (crypto_kx_client_session_keys(rx, tx, pk, sk, peer_pk) != 0) return -1;
    memcpy(state->session_key, rx, ALLOY_IPC_KEY_SIZE);
    state->next_outbound_seq = 1;
    state->last_inbound_seq = 0;
    state->transport_fd = transport_fd;
    return 0;
}

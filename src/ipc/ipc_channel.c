/**
 * AlloyScript — Secure IPC Channel (libsodium)
 *
 * Transport: Unix domain socket (macOS/Linux) / named pipe (Windows).
 * Handshake: X25519 key exchange via crypto_kx.
 * Per-message: XChaCha20-Poly1305 (crypto_secretstream) with sequence numbers.
 * Wire format: AlloyWireHeader (24 bytes, plaintext) + encrypted payload.
 *
 * The main core is the server (trusted). The hidden web process is the client.
 */
#include "../../include/alloy/ipc_proto.h"
#include <sodium.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>

#ifdef _WIN32
#  include <windows.h>
#  define ALLOY_PIPE_PREFIX "\\\\.\\pipe\\alloy_"
typedef HANDLE alloy_fd_t;
#  define ALLOY_INVALID_FD INVALID_HANDLE_VALUE
#else
#  include <sys/socket.h>
#  include <sys/un.h>
#  include <unistd.h>
#  include <fcntl.h>
#  include <poll.h>
typedef int alloy_fd_t;
#  define ALLOY_INVALID_FD (-1)
#endif

/* ── Channel struct ─────────────────────────────────────────────────────── */
struct AlloyIPCChannel {
    alloy_fd_t       fd;
    AlloyIPCSession  session;
    bool             is_server;
    char             path[256];

    /* libsodium secretstream state */
    crypto_secretstream_xchacha20poly1305_state tx_state;
    crypto_secretstream_xchacha20poly1305_state rx_state;
};

/* ── Platform I/O helpers ───────────────────────────────────────────────── */

static int fd_write_all(alloy_fd_t fd, const uint8_t *buf, size_t len) {
#ifdef _WIN32
    DWORD written;
    return WriteFile(fd, buf, (DWORD)len, &written, NULL) && written == len ? 0 : -1;
#else
    size_t off = 0;
    while (off < len) {
        ssize_t n = write(fd, buf + off, len - off);
        if (n <= 0) return -1;
        off += (size_t)n;
    }
    return 0;
#endif
}

static int fd_read_all(alloy_fd_t fd, uint8_t *buf, size_t len) {
#ifdef _WIN32
    DWORD got;
    return ReadFile(fd, buf, (DWORD)len, &got, NULL) && got == len ? 0 : -1;
#else
    size_t off = 0;
    while (off < len) {
        ssize_t n = read(fd, buf + off, len - off);
        if (n <= 0) return -1;
        off += (size_t)n;
    }
    return 0;
#endif
}

/* ── Socket creation ────────────────────────────────────────────────────── */

#ifndef _WIN32
static alloy_fd_t unix_server_listen(const char *path) {
    alloy_fd_t srv = socket(AF_UNIX, SOCK_STREAM, 0);
    if (srv < 0) return ALLOY_INVALID_FD;

    struct sockaddr_un addr;
    memset(&addr, 0, sizeof(addr));
    addr.sun_family = AF_UNIX;
    strncpy(addr.sun_path, path, sizeof(addr.sun_path) - 1);
    unlink(path);  /* remove stale socket */

    if (bind(srv, (struct sockaddr *)&addr, sizeof(addr)) < 0 ||
        listen(srv, 1) < 0) {
        close(srv);
        return ALLOY_INVALID_FD;
    }
    return srv;
}

static alloy_fd_t unix_server_accept(alloy_fd_t srv) {
    return accept(srv, NULL, NULL);
}

static alloy_fd_t unix_client_connect(const char *path) {
    alloy_fd_t fd = socket(AF_UNIX, SOCK_STREAM, 0);
    if (fd < 0) return ALLOY_INVALID_FD;

    struct sockaddr_un addr;
    memset(&addr, 0, sizeof(addr));
    addr.sun_family = AF_UNIX;
    strncpy(addr.sun_path, path, sizeof(addr.sun_path) - 1);

    /* Retry a few times — server may not be ready yet */
    for (int i = 0; i < 20; i++) {
        if (connect(fd, (struct sockaddr *)&addr, sizeof(addr)) == 0) return fd;
        usleep(50000);  /* 50 ms */
    }
    close(fd);
    return ALLOY_INVALID_FD;
}
#endif

/* ── Public constructors ────────────────────────────────────────────────── */

AlloyIPCChannel *alloy_ipc_server_create(const char *path) {
    if (sodium_init() < 0) return NULL;

    AlloyIPCChannel *ch = calloc(1, sizeof(AlloyIPCChannel));
    if (!ch) return NULL;
    ch->is_server = true;
    strncpy(ch->path, path, sizeof(ch->path) - 1);

#ifndef _WIN32
    alloy_fd_t srv = unix_server_listen(path);
    if (srv == ALLOY_INVALID_FD) { free(ch); return NULL; }
    ch->fd = unix_server_accept(srv);
    close(srv);
    if (ch->fd == ALLOY_INVALID_FD) { free(ch); return NULL; }
#else
    /* Windows named pipe — simplified */
    char pipe_name[300];
    snprintf(pipe_name, sizeof(pipe_name), "%s%s", ALLOY_PIPE_PREFIX, path);
    ch->fd = CreateNamedPipeA(pipe_name,
        PIPE_ACCESS_DUPLEX, PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
        1, 65536, 65536, 0, NULL);
    if (ch->fd == ALLOY_INVALID_FD) { free(ch); return NULL; }
    ConnectNamedPipe(ch->fd, NULL);
#endif
    return ch;
}

AlloyIPCChannel *alloy_ipc_client_connect(const char *path) {
    if (sodium_init() < 0) return NULL;

    AlloyIPCChannel *ch = calloc(1, sizeof(AlloyIPCChannel));
    if (!ch) return NULL;
    ch->is_server = false;
    strncpy(ch->path, path, sizeof(ch->path) - 1);

#ifndef _WIN32
    ch->fd = unix_client_connect(path);
#else
    char pipe_name[300];
    snprintf(pipe_name, sizeof(pipe_name), "%s%s", ALLOY_PIPE_PREFIX, path);
    ch->fd = CreateFileA(pipe_name, GENERIC_READ | GENERIC_WRITE, 0, NULL,
                         OPEN_EXISTING, 0, NULL);
#endif
    if (ch->fd == ALLOY_INVALID_FD) { free(ch); return NULL; }
    return ch;
}

void alloy_ipc_channel_destroy(AlloyIPCChannel *ch) {
    if (!ch) return;
#ifdef _WIN32
    if (ch->fd != ALLOY_INVALID_FD) CloseHandle(ch->fd);
#else
    if (ch->fd != ALLOY_INVALID_FD) close(ch->fd);
    if (ch->is_server) unlink(ch->path);
#endif
    sodium_memzero(&ch->session, sizeof(ch->session));
    sodium_memzero(&ch->tx_state, sizeof(ch->tx_state));
    sodium_memzero(&ch->rx_state, sizeof(ch->rx_state));
    free(ch);
}

/* ── X25519 Handshake ───────────────────────────────────────────────────── */

int alloy_ipc_handshake(AlloyIPCChannel *ch, bool is_server) {
    uint8_t my_pk[ALLOY_IPC_X25519_PK_BYTES];
    uint8_t my_sk[ALLOY_IPC_X25519_SK_BYTES];
    uint8_t peer_pk[ALLOY_IPC_X25519_PK_BYTES];

    crypto_kx_keypair(my_pk, my_sk);

    if (is_server) {
        /* Server: send pubkey first, then receive client pubkey */
        if (fd_write_all(ch->fd, my_pk, sizeof(my_pk)) < 0) goto fail;
        if (fd_read_all(ch->fd, peer_pk, sizeof(peer_pk)) < 0) goto fail;

        uint8_t rx[crypto_kx_SESSIONKEYBYTES];
        uint8_t tx[crypto_kx_SESSIONKEYBYTES];
        if (crypto_kx_server_session_keys(rx, tx, my_pk, my_sk, peer_pk) != 0) goto fail;
        memcpy(ch->session.rx_key, rx, ALLOY_IPC_KEY_BYTES);
        memcpy(ch->session.tx_key, tx, ALLOY_IPC_KEY_BYTES);
    } else {
        /* Client: receive server pubkey first, then send ours */
        if (fd_read_all(ch->fd, peer_pk, sizeof(peer_pk)) < 0) goto fail;
        if (fd_write_all(ch->fd, my_pk, sizeof(my_pk)) < 0) goto fail;

        uint8_t rx[crypto_kx_SESSIONKEYBYTES];
        uint8_t tx[crypto_kx_SESSIONKEYBYTES];
        if (crypto_kx_client_session_keys(rx, tx, my_pk, my_sk, peer_pk) != 0) goto fail;
        memcpy(ch->session.rx_key, rx, ALLOY_IPC_KEY_BYTES);
        memcpy(ch->session.tx_key, tx, ALLOY_IPC_KEY_BYTES);
    }

    /* Initialise secretstream states */
    uint8_t tx_header[crypto_secretstream_xchacha20poly1305_HEADERBYTES];
    crypto_secretstream_xchacha20poly1305_init_push(&ch->tx_state, tx_header, ch->session.tx_key);
    if (fd_write_all(ch->fd, tx_header, sizeof(tx_header)) < 0) goto fail;

    uint8_t rx_header[crypto_secretstream_xchacha20poly1305_HEADERBYTES];
    if (fd_read_all(ch->fd, rx_header, sizeof(rx_header)) < 0) goto fail;
    if (crypto_secretstream_xchacha20poly1305_init_pull(&ch->rx_state, rx_header, ch->session.rx_key) != 0) goto fail;

    ch->session.established = true;
    sodium_memzero(my_sk, sizeof(my_sk));
    return 0;

fail:
    sodium_memzero(my_sk, sizeof(my_sk));
    return -1;
}

/* ── Send / Receive ─────────────────────────────────────────────────────── */

int alloy_ipc_send(AlloyIPCChannel *ch, AlloyMsgType type,
                   const uint8_t *payload, uint32_t payload_len,
                   const char *cap_id) {
    if (!ch || !ch->session.established) return -1;
    if (payload_len > ALLOY_IPC_MAX_PAYLOAD) return -1;

    /* Build plaintext: 1-byte type + 8-byte seq + 32-byte cap_id + payload */
    uint64_t seq = ++ch->session.tx_seq;
    size_t pt_len = 1 + 8 + 32 + payload_len;
    uint8_t *pt = malloc(pt_len);
    if (!pt) return -1;

    pt[0] = (uint8_t)type;
    memcpy(pt + 1, &seq, 8);
    if (cap_id) {
        size_t clen = strlen(cap_id);
        memcpy(pt + 9, cap_id, clen < 32 ? clen : 32);
    }
    if (payload && payload_len > 0)
        memcpy(pt + 41, payload, payload_len);

    /* Encrypt */
    size_t ct_len = pt_len + crypto_secretstream_xchacha20poly1305_ABYTES;
    uint8_t *ct = malloc(ct_len);
    if (!ct) { free(pt); return -1; }

    crypto_secretstream_xchacha20poly1305_push(
        &ch->tx_state, ct, NULL, pt, pt_len, NULL, 0,
        crypto_secretstream_xchacha20poly1305_TAG_MESSAGE);
    free(pt);

    /* Write length-prefixed frame */
    uint32_t frame_len = (uint32_t)ct_len;
    if (fd_write_all(ch->fd, (uint8_t *)&frame_len, 4) < 0 ||
        fd_write_all(ch->fd, ct, ct_len) < 0) {
        free(ct);
        return -1;
    }
    free(ct);
    return 0;
}

int alloy_ipc_recv(AlloyIPCChannel *ch, AlloyMsg *msg) {
    if (!ch || !ch->session.established || !msg) return -1;

    /* Read frame length */
    uint32_t frame_len = 0;
    if (fd_read_all(ch->fd, (uint8_t *)&frame_len, 4) < 0) return -1;
    if (frame_len > ALLOY_IPC_MAX_PAYLOAD + 256) return -1;

    uint8_t *ct = malloc(frame_len);
    if (!ct) return -1;
    if (fd_read_all(ch->fd, ct, frame_len) < 0) { free(ct); return -1; }

    /* Decrypt */
    size_t pt_len = frame_len - crypto_secretstream_xchacha20poly1305_ABYTES;
    uint8_t *pt = malloc(pt_len + 1);
    if (!pt) { free(ct); return -1; }

    uint8_t tag;
    if (crypto_secretstream_xchacha20poly1305_pull(
            &ch->rx_state, pt, NULL, &tag, ct, frame_len, NULL, 0) != 0) {
        free(ct); free(pt);
        fprintf(stderr, "[alloy:ipc] decryption failed — possible tampering\n");
        return -1;
    }
    free(ct);

    /* Parse plaintext: 1-byte type + 8-byte seq + 32-byte cap_id + payload */
    if (pt_len < 41) { free(pt); return -1; }

    msg->type = (AlloyMsgType)pt[0];
    memcpy(&msg->seq, pt + 1, 8);
    memcpy(msg->cap_id, pt + 9, 32);
    msg->cap_id[32] = '\0';

    /* Replay check */
    if (msg->seq <= ch->session.rx_seq) {
        fprintf(stderr, "[alloy:ipc] replay detected (seq %llu)\n",
                (unsigned long long)msg->seq);
        free(pt);
        return -1;
    }
    ch->session.rx_seq = msg->seq;

    msg->payload_len = (uint32_t)(pt_len - 41);
    msg->payload = malloc(msg->payload_len + 1);
    if (!msg->payload) { free(pt); return -1; }
    memcpy(msg->payload, pt + 41, msg->payload_len);
    msg->payload[msg->payload_len] = '\0';
    free(pt);
    return 0;
}

int alloy_ipc_poll(AlloyIPCChannel *ch) {
    if (!ch || ch->fd == ALLOY_INVALID_FD) return -1;
#ifdef _WIN32
    DWORD avail = 0;
    return PeekNamedPipe(ch->fd, NULL, 0, NULL, &avail, NULL) && avail > 0 ? 1 : 0;
#else
    struct pollfd pfd = { .fd = ch->fd, .events = POLLIN };
    int r = poll(&pfd, 1, 0);
    return r > 0 ? 1 : (r == 0 ? 0 : -1);
#endif
}

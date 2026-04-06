/**
 * AlloyScript Phase 1 — Core Tests
 *
 * Tests:
 *   1. MicroQuickJS embedding + eval round-trip
 *   2. Capability store: issue, validate, deny, revoke
 *   3. IPC channel: loopback encrypt/decrypt
 *   4. Engine vtable: call, eval, bind, emit, destroy safety
 */
#include "../include/alloy/js_engine.h"
#include "../include/alloy/capability.h"
#include "../include/alloy/ipc_proto.h"
#include "../packages/mquickjs/mquickjs.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>
#include <pthread.h>

/* Forward declarations */
AlloyJSEngine *alloy_mqjs_adapter_create(JSContext *ctx);
const JSSTDLibraryDef *alloy_get_stdlib(void);

/* ── Test helpers ───────────────────────────────────────────────────────── */
static int g_pass = 0, g_fail = 0;

#define TEST(name) do { \
    printf("  %-50s", name); fflush(stdout); \
} while(0)

#define PASS() do { g_pass++; printf("PASS\n"); } while(0)
#define FAIL(msg) do { g_fail++; printf("FAIL: %s\n", msg); } while(0)

#define ASSERT_STR_CONTAINS(haystack, needle) do { \
    if (!haystack || !strstr(haystack, needle)) { \
        FAIL("expected '" needle "' in: " #haystack); \
        return; \
    } \
} while(0)

#define ASSERT_TRUE(cond) do { \
    if (!(cond)) { FAIL(#cond " was false"); return; } \
} while(0)

/* ── Test 1: MicroQuickJS eval round-trip ───────────────────────────────── */
static void test_mqjs_eval(void) {
    TEST("mqjs: eval number");
    static uint8_t heap[256 * 1024];
    JSContext *ctx = JS_NewContext(heap, sizeof(heap), alloy_get_stdlib());
    assert(ctx);

    AlloyJSEngine *eng = alloy_mqjs_adapter_create(ctx);
    assert(eng);

    char *r = alloy_engine_eval(eng, "1 + 2", "<test>");
    ASSERT_STR_CONTAINS(r, "3");
    free(r);
    PASS();

    TEST("mqjs: eval string");
    r = alloy_engine_eval(eng, "'hello' + ' world'", "<test>");
    ASSERT_STR_CONTAINS(r, "hello world");
    free(r);
    PASS();

    TEST("mqjs: eval object");
    r = alloy_engine_eval(eng, "({x:1,y:2})", "<test>");
    ASSERT_STR_CONTAINS(r, "\"x\"");
    free(r);
    PASS();

    TEST("mqjs: eval exception serialised");
    r = alloy_engine_eval(eng, "throw new Error('boom')", "<test>");
    ASSERT_STR_CONTAINS(r, "error");
    free(r);
    PASS();

    alloy_engine_destroy(eng);
}

/* ── Test 2: bind + call ────────────────────────────────────────────────── */
static char *echo_handler(const char *name, const char *args_json, void *ud) {
    (void)name; (void)ud;
    /* Return the first argument as-is */
    char *out = NULL;
    asprintf(&out, "{\"echo\":%s}", args_json);
    return out;
}

static void test_mqjs_bind_call(void) {
    TEST("mqjs: bind + call");
    static uint8_t heap[256 * 1024];
    JSContext *ctx = JS_NewContext(heap, sizeof(heap), alloy_get_stdlib());
    AlloyJSEngine *eng = alloy_mqjs_adapter_create(ctx);

    int r = alloy_engine_bind(eng, "__test_echo", echo_handler, NULL);
    ASSERT_TRUE(r == ALLOY_OK);

    char *result = alloy_engine_eval(eng, "__test_echo(42)", "<test>");
    ASSERT_STR_CONTAINS(result, "42");
    free(result);
    PASS();

    alloy_engine_destroy(eng);
}

/* ── Test 3: use-after-destroy safety ───────────────────────────────────── */
static void test_use_after_destroy(void) {
    TEST("vtable: use-after-destroy returns NULL/error");
    static uint8_t heap[256 * 1024];
    JSContext *ctx = JS_NewContext(heap, sizeof(heap), alloy_get_stdlib());
    AlloyJSEngine *eng = alloy_mqjs_adapter_create(ctx);

    alloy_engine_destroy(eng);

    /* All vtable wrappers must be safe after destroy */
    char *r1 = alloy_engine_eval(eng, "1+1", "<test>");
    char *r2 = alloy_engine_call(eng, "foo", "[]");
    int   r3 = alloy_engine_bind(eng, "x", echo_handler, NULL);
    int   r4 = alloy_engine_emit(eng, "ev", "null");

    ASSERT_TRUE(r1 == NULL);
    ASSERT_TRUE(r2 == NULL);
    ASSERT_TRUE(r3 == ALLOY_ERR_DESTROYED);
    ASSERT_TRUE(r4 == ALLOY_ERR_DESTROYED);
    PASS();
}

/* ── Test 4: Capability store ───────────────────────────────────────────── */
static void test_capability_store(void) {
    TEST("cap: issue + validate");
    AlloyCapStore *store = alloy_cap_store_create();
    assert(store);

    const char *scopes[] = { "https://api.example.com/*" };
    char *cap_id = alloy_cap_issue(store, "web:fetch", ALLOY_PERM_NETWORK,
                                   scopes, 1, "alloy:web", 0);
    ASSERT_TRUE(cap_id != NULL);
    ASSERT_TRUE(strlen(cap_id) == 32);

    bool ok = alloy_cap_validate(store, cap_id, "web:fetch", ALLOY_PERM_NETWORK,
                                  "https://api.example.com/v1/data");
    ASSERT_TRUE(ok);
    free(cap_id);
    PASS();

    TEST("cap: wrong resource denied");
    cap_id = alloy_cap_issue(store, "web:fetch", ALLOY_PERM_NETWORK,
                              scopes, 1, "alloy:web", 0);
    bool denied = alloy_cap_validate(store, cap_id, "fs:read", ALLOY_PERM_READ, NULL);
    ASSERT_TRUE(!denied);
    free(cap_id);
    PASS();

    TEST("cap: wrong scope denied");
    cap_id = alloy_cap_issue(store, "web:fetch", ALLOY_PERM_NETWORK,
                              scopes, 1, "alloy:web", 0);
    bool scope_denied = alloy_cap_validate(store, cap_id, "web:fetch",
                                            ALLOY_PERM_NETWORK,
                                            "https://evil.com/steal");
    ASSERT_TRUE(!scope_denied);
    free(cap_id);
    PASS();

    TEST("cap: revoke");
    cap_id = alloy_cap_issue(store, "web:fetch", ALLOY_PERM_NETWORK,
                              scopes, 1, "alloy:web", 0);
    alloy_cap_revoke(store, cap_id);
    bool revoked = alloy_cap_validate(store, cap_id, "web:fetch",
                                       ALLOY_PERM_NETWORK, NULL);
    ASSERT_TRUE(!revoked);
    free(cap_id);
    PASS();

    TEST("cap: expired");
    char *exp_cap = alloy_cap_issue(store, "web:fetch", ALLOY_PERM_NETWORK,
                                     NULL, 0, "alloy:web", 1); /* expired in 1970 */
    bool expired = alloy_cap_validate(store, exp_cap, "web:fetch",
                                       ALLOY_PERM_NETWORK, NULL);
    ASSERT_TRUE(!expired);
    free(exp_cap);
    PASS();

    alloy_cap_store_destroy(store);
}

/* ── Test 5: IPC loopback ───────────────────────────────────────────────── */
typedef struct { const char *path; } IPCServerArg;

static void *ipc_server_thread(void *arg) {
    IPCServerArg *a = (IPCServerArg *)arg;
    AlloyIPCChannel *srv = alloy_ipc_server_create(a->path);
    if (!srv) return (void *)-1;
    alloy_ipc_handshake(srv, true);

    AlloyMsg msg;
    if (alloy_ipc_recv(srv, &msg) == 0) {
        /* Echo back */
        alloy_ipc_send(srv, ALLOY_MSG_WEB_API_RESPONSE,
                       msg.payload, msg.payload_len, "");
        free(msg.payload);
    }
    alloy_ipc_channel_destroy(srv);
    return NULL;
}

static void test_ipc_loopback(void) {
    TEST("ipc: loopback encrypt/decrypt");

    static const char *path = "/tmp/alloy_test_ipc.sock";
    IPCServerArg arg = { path };

    pthread_t thr;
    pthread_create(&thr, NULL, ipc_server_thread, &arg);
    usleep(50000);  /* give server time to bind */

    AlloyIPCChannel *cli = alloy_ipc_client_connect(path);
    ASSERT_TRUE(cli != NULL);
    alloy_ipc_handshake(cli, false);

    const char *payload = "{\"method\":\"fetch\",\"args\":[]}";
    int r = alloy_ipc_send(cli, ALLOY_MSG_WEB_API_REQUEST,
                            (uint8_t *)payload, (uint32_t)strlen(payload), "");
    ASSERT_TRUE(r == 0);

    AlloyMsg resp;
    r = alloy_ipc_recv(cli, &resp);
    ASSERT_TRUE(r == 0);
    ASSERT_TRUE(resp.payload != NULL);
    ASSERT_TRUE(strstr((char *)resp.payload, "fetch") != NULL);
    free(resp.payload);

    alloy_ipc_channel_destroy(cli);
    pthread_join(thr, NULL);
    PASS();
}

/* ── Test 6: emit ───────────────────────────────────────────────────────── */
static void test_emit(void) {
    TEST("mqjs: emit dispatches __alloy_emit");
    static uint8_t heap[256 * 1024];
    JSContext *ctx = JS_NewContext(heap, sizeof(heap), alloy_get_stdlib());
    AlloyJSEngine *eng = alloy_mqjs_adapter_create(ctx);

    /* Install a listener */
    alloy_engine_eval(eng,
        "var __alloy_emit_received = null;"
        "function __alloy_emit(ev, payload) { __alloy_emit_received = ev; }",
        "<test>");

    int r = alloy_engine_emit(eng, "test_event", "{\"x\":1}");
    ASSERT_TRUE(r == ALLOY_OK);

    char *got = alloy_engine_eval(eng, "__alloy_emit_received", "<test>");
    ASSERT_STR_CONTAINS(got, "test_event");
    free(got);
    PASS();

    alloy_engine_destroy(eng);
}

/* ── Main ───────────────────────────────────────────────────────────────── */
int main(void) {
    printf("\nAlloyScript Phase 1 — Core Tests\n");
    printf("=================================\n\n");

    test_mqjs_eval();
    test_mqjs_bind_call();
    test_use_after_destroy();
    test_capability_store();
    test_ipc_loopback();
    test_emit();

    printf("\n=================================\n");
    printf("Results: %d passed, %d failed\n\n", g_pass, g_fail);
    return g_fail > 0 ? 1 : 0;
}

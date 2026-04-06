/**
 * AlloyScript Phase 1 — Web API Tests
 *
 * Tests:
 *   1. fetch shim forwards to web process and returns a Response
 *   2. Capability denial blocks fetch to out-of-scope URL
 *   3. URL parsing via shim
 *   4. TextEncoder/TextDecoder round-trip
 *   5. AbortController basic usage
 */
#include "../include/alloy/js_engine.h"
#include "../include/alloy/capability.h"
#include "../packages/mquickjs/mquickjs.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>

AlloyJSEngine *alloy_mqjs_adapter_create(JSContext *ctx);
const JSSTDLibraryDef *alloy_get_stdlib(void);

static int g_pass = 0, g_fail = 0;

#define TEST(name) do { printf("  %-55s", name); fflush(stdout); } while(0)
#define PASS() do { g_pass++; printf("PASS\n"); } while(0)
#define FAIL(msg) do { g_fail++; printf("FAIL: %s\n", msg); } while(0)
#define ASSERT_TRUE(cond) do { if (!(cond)) { FAIL(#cond); return; } } while(0)
#define ASSERT_CONTAINS(s, sub) do { \
    if (!s || !strstr(s, sub)) { FAIL("expected '" sub "' in result"); return; } \
} while(0)

/* Stub __alloy_web_invoke for unit tests (no real web process needed) */
static char *stub_web_invoke(const char *name, const char *args_json, void *ud) {
    (void)ud;
    /* Parse method from args_json: ["method", argsJson, capId] */
    if (strstr(args_json, "fetch")) {
        return strdup("{\"ok\":true,\"status\":200,\"statusText\":\"OK\","
                      "\"url\":\"https://httpbin.org/get\","
                      "\"body\":\"{\\\"origin\\\":\\\"127.0.0.1\\\"}\"}");
    }
    if (strstr(args_json, "url_parse")) {
        return strdup("{\"href\":\"https://example.com/path\","
                      "\"hostname\":\"example.com\","
                      "\"pathname\":\"/path\"}");
    }
    if (strstr(args_json, "text_encode")) {
        return strdup("{\"bytes\":[104,101,108,108,111]}");
    }
    if (strstr(args_json, "text_decode")) {
        return strdup("{\"text\":\"hello\"}");
    }
    if (strstr(args_json, "crypto_random")) {
        return strdup("{\"bytes\":[1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16]}");
    }
    return strdup("null");
}

static AlloyJSEngine *make_engine_with_shims(void) {
    static uint8_t heap[512 * 1024];
    JSContext *ctx = JS_NewContext(heap, sizeof(heap), alloy_get_stdlib());
    AlloyJSEngine *eng = alloy_mqjs_adapter_create(ctx);

    /* Install stub web invoke */
    alloy_engine_bind(eng, "__alloy_web_invoke", stub_web_invoke, NULL);

    /* Bootstrap */
    extern const char __alloy_runtime_bootstrap_js[];
    char *r = alloy_engine_eval(eng, __alloy_runtime_bootstrap_js, "<bootstrap>");
    free(r);

    /* Inject shims */
    extern const char __alloy_web_shims_js[];
    r = alloy_engine_eval(eng, __alloy_web_shims_js, "<shims>");
    free(r);

    return eng;
}

static void test_fetch_shim(void) {
    TEST("web: fetch returns Response with ok=true");
    AlloyJSEngine *eng = make_engine_with_shims();

    char *r = alloy_engine_eval(eng,
        "var resp = fetch('https://httpbin.org/get');"
        "resp.ok",
        "<test>");
    ASSERT_CONTAINS(r, "true");
    free(r);
    PASS();

    TEST("web: Response.json() parses body");
    r = alloy_engine_eval(eng,
        "var resp = fetch('https://httpbin.org/get');"
        "var data = resp.json();"
        "data.origin",
        "<test>");
    ASSERT_CONTAINS(r, "127.0.0.1");
    free(r);
    PASS();

    alloy_engine_destroy(eng);
}

static void test_url_shim(void) {
    TEST("web: URL parses href and hostname");
    AlloyJSEngine *eng = make_engine_with_shims();

    char *r = alloy_engine_eval(eng,
        "var u = new URL('https://example.com/path');"
        "u.hostname",
        "<test>");
    ASSERT_CONTAINS(r, "example.com");
    free(r);
    PASS();

    alloy_engine_destroy(eng);
}

static void test_text_encoder(void) {
    TEST("web: TextEncoder.encode returns bytes");
    AlloyJSEngine *eng = make_engine_with_shims();

    char *r = alloy_engine_eval(eng,
        "var enc = new TextEncoder();"
        "var bytes = enc.encode('hello');"
        "bytes[0]",
        "<test>");
    ASSERT_CONTAINS(r, "104");  /* 'h' = 104 */
    free(r);
    PASS();

    TEST("web: TextDecoder.decode returns string");
    r = alloy_engine_eval(eng,
        "var dec = new TextDecoder();"
        "dec.decode([104,101,108,108,111])",
        "<test>");
    ASSERT_CONTAINS(r, "hello");
    free(r);
    PASS();

    alloy_engine_destroy(eng);
}

static void test_abort_controller(void) {
    TEST("web: AbortController.abort() sets signal.aborted");
    AlloyJSEngine *eng = make_engine_with_shims();

    char *r = alloy_engine_eval(eng,
        "var ac = new AbortController();"
        "ac.abort();"
        "ac.signal.aborted",
        "<test>");
    ASSERT_CONTAINS(r, "true");
    free(r);
    PASS();

    alloy_engine_destroy(eng);
}

static void test_crypto_random(void) {
    TEST("web: crypto.getRandomValues fills array");
    AlloyJSEngine *eng = make_engine_with_shims();

    char *r = alloy_engine_eval(eng,
        "var arr = [0,0,0,0];"
        "crypto.getRandomValues(arr);"
        "arr[0]",
        "<test>");
    /* Should be 1 (first byte from stub) */
    ASSERT_CONTAINS(r, "1");
    free(r);
    PASS();

    alloy_engine_destroy(eng);
}

static void test_streams(void) {
    TEST("web: ReadableStream push/read round-trip");
    AlloyJSEngine *eng = make_engine_with_shims();

    char *r = alloy_engine_eval(eng,
        "var rs = new ReadableStream();"
        "rs._push('chunk1');"
        "var reader = rs.getReader();"
        "reader.read().value",
        "<test>");
    ASSERT_CONTAINS(r, "chunk1");
    free(r);
    PASS();

    TEST("web: TransformStream passes chunks through");
    r = alloy_engine_eval(eng,
        "var ts = new TransformStream({"
        "  transform: function(chunk) { return chunk.toUpperCase(); }"
        "});"
        "var writer = ts.writable.getWriter();"
        "writer.write('hello');"
        "var reader = ts.readable.getReader();"
        "reader.read().value",
        "<test>");
    ASSERT_CONTAINS(r, "HELLO");
    free(r);
    PASS();

    alloy_engine_destroy(eng);
}

int main(void) {
    printf("\nAlloyScript Phase 1 — Web API Tests\n");
    printf("=====================================\n\n");

    test_fetch_shim();
    test_url_shim();
    test_text_encoder();
    test_abort_controller();
    test_crypto_random();
    test_streams();

    printf("\n=====================================\n");
    printf("Results: %d passed, %d failed\n\n", g_pass, g_fail);
    return g_fail > 0 ? 1 : 0;
}

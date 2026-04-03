#include "shell.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// production-ready JSON helper (skeletal but safer)
static char *get_json_string(const char *json, const char *key) {
    char search_key[128];
    snprintf(search_key, sizeof(search_key), "\"%s\":", key);
    char *p = strstr(json, search_key);
    if (!p) return NULL;
    p = strchr(p, ':');
    if (!p) return NULL;
    p = strchr(p, '"');
    if (!p) return NULL;
    char *end = strchr(p + 1, '"');
    if (!end) return NULL;
    return strndup(p + 1, end - (p + 1));
}

static char *base64_encode(const uint8_t *input, size_t length) {
    static const char table[] = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    size_t out_len = 4 * ((length + 2) / 3);
    char *out = malloc(out_len + 1);
    if (!out) return NULL;
    size_t i = 0, j = 0;
    while (i < length) {
        uint32_t octet_a = i < length ? input[i++] : 0;
        uint32_t octet_b = i < length ? input[i++] : 0;
        uint32_t octet_c = i < length ? input[i++] : 0;
        uint32_t triple = (octet_a << 16) | (octet_b << 8) | octet_c;
        out[j++] = table[(triple >> 18) & 0x3F];
        out[j++] = table[(triple >> 12) & 0x3F];
        out[j++] = table[(triple >> 6) & 0x3F];
        out[j++] = table[triple & 0x3F];
    }
    if (length % 3 >= 1) out[out_len - 1] = '=';
    if (length % 3 == 1) out[out_len - 2] = '=';
    out[out_len] = '\0';
    return out;
}

int main(int argc, char **argv) {
    char buf[65536];
    if (!fgets(buf, sizeof(buf), stdin)) return 1;

    char *script = get_json_string(buf, "script");
    if (!script) return 1;

    alloy_shell_ctx_t ctx;
    memset(&ctx, 0, sizeof(ctx));
    alloy_shell_result_t result;
    memset(&result, 0, sizeof(result));

    if (alloy_shell_exec(&ctx, script, &result) < 0) {
        free(script);
        return 1;
    }

    char *out_b64 = base64_encode(result.stdout_buf, result.stdout_len);
    char *err_b64 = base64_encode(result.stderr_buf, result.stderr_len);

    printf("{\"exitCode\": %d, \"stdout\": \"%s\", \"stderr\": \"%s\"}\n",
           result.exit_code, out_b64 ? out_b64 : "", err_b64 ? err_b64 : "");

    free(out_b64); free(err_b64);
    free(result.stdout_buf); free(result.stderr_buf);
    free(script);
    return 0;
}

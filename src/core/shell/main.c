#include "shell.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// Mock implementation for alloy_shell_exec as it's not fully defined in interpreter.c yet
extern int alloy_shell_exec(alloy_shell_ctx_t *ctx, const char *script, alloy_shell_result_t *out);

// Base64 functions
static char *base64_encode(const uint8_t *input, int length);

int main(int argc, char **argv) {
    char buf[4096];
    if (!fgets(buf, sizeof(buf), stdin)) return 1;

    // JSON Parsing - simplified
    char *script_ptr = strstr(buf, "\"script\":");
    if (!script_ptr) return 1;
    script_ptr = strchr(script_ptr, '"') + 1;
    char *script_end = strchr(script_ptr + 8, '"');
    if (!script_end) return 1;
    *script_end = '\0';

    alloy_shell_ctx_t ctx = { {1, NULL, NULL}, NULL, 0 };
    alloy_shell_result_t result;
    alloy_shell_exec(&ctx, script_ptr, &result);

    char *stdout_base64 = base64_encode(result.stdout_buf, result.stdout_len);
    char *stderr_base64 = base64_encode(result.stderr_buf, result.stderr_len);

    printf("{\"exitCode\": %d, \"stdout\": \"%s\", \"stderr\": \"%s\"}\n",
           result.exit_code, stdout_base64, stderr_base64);

    free(stdout_base64); free(stderr_base64);
    free(result.stdout_buf); free(result.stderr_buf);

    return 0;
}

static char *base64_encode(const uint8_t *input, int length) {
    static const char table[] = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    int out_len = 4 * ((length + 2) / 3);
    char *out = malloc(out_len + 1);
    int i = 0, j = 0;
    while (i < length) {
        uint32_t triple = (input[i++] << 16);
        if (i < length) triple |= (input[i++] << 8);
        if (i < length) triple |= input[i++];
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

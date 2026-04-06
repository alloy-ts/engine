#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <unistd.h>
#include <fcntl.h>
#include "lexer.h"
#include "parser.h"
#include "builtins.h"

typedef struct { uint8_t *stdout_buf; size_t stdout_len; uint8_t *stderr_buf; size_t stderr_len; int exit_code; } alloy_shell_result_t;
extern int alloy_shell_exec(alloy_shell_ctx_t *ctx, alloy_node_t *script, alloy_shell_result_t *res);

static char *base64_encode(const uint8_t *input, int length) {
    static const char table[] = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    int out_len = 4 * ((length + 2) / 3);
    char *out = malloc(out_len + 1);
    for (int i = 0, j = 0; i < length;) {
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
    char *buf = malloc(1024 * 1024); if (!buf) return 1;
    ssize_t n = read(STDIN_FILENO, buf, 1024 * 1024 - 1); if (n < 0) { free(buf); return 1; } buf[n] = '\0';

    char *script_text = NULL;
    // Minimal JSON parsing for {"script": "..."}
    char *p = strstr(buf, "\"script\":");
    if (p) {
        p += 9;
        while (*p && (*p == ' ' || *p == ':')) p++;
        if (*p == '"') {
            p++;
            script_text = p;
            int escaped = 0;
            while (*p) {
                if (*p == '"' && !escaped) { *p = '\0'; break; }
                if (*p == '\\' && !escaped) escaped = 1;
                else escaped = 0;
                p++;
            }
        }
    }

    if (!script_text) script_text = buf;

    // Handle JSON escapes in script_text
    char *unescaped = malloc(strlen(script_text) + 1);
    int j = 0;
    for (int i = 0; script_text[i]; i++) {
        if (script_text[i] == '\\' && (script_text[i+1] == '"' || script_text[i+1] == '\\')) {
            unescaped[j++] = script_text[++i];
        } else {
            unescaped[j++] = script_text[i];
        }
    }
    unescaped[j] = '\0';

    alloy_token_list_t tl = {0};
    if (alloy_shell_lex(unescaped, &tl) != 0) { free(buf); free(unescaped); return 1; }

    alloy_node_t *root = NULL;
    if (alloy_shell_parse(&tl, &root) != 0) { alloy_token_list_free(&tl); free(buf); free(unescaped); return 1; }

    // Capture stdout/stderr to files
    char stdout_file[] = "/tmp/alloy_stdout_XXXXXX";
    char stderr_file[] = "/tmp/alloy_stderr_XXXXXX";
    int fd_out = mkstemp(stdout_file);
    int fd_err = mkstemp(stderr_file);

    alloy_shell_ctx_t ctx = {0};
    alloy_shell_result_t res = {0};
    alloy_stdio_t io = { STDIN_FILENO, fd_out, fd_err };

    alloy_shell_exec(&ctx, root, &res);

    // Read them back
    lseek(fd_out, 0, SEEK_SET);
    res.stdout_buf = malloc(1024 * 1024);
    res.stdout_len = read(fd_out, res.stdout_buf, 1024 * 1024);
    close(fd_out); unlink(stdout_file);

    lseek(fd_err, 0, SEEK_SET);
    res.stderr_buf = malloc(1024 * 1024);
    res.stderr_len = read(fd_err, res.stderr_buf, 1024 * 1024);
    close(fd_err); unlink(stderr_file);

    char *out_b64 = base64_encode(res.stdout_buf, res.stdout_len > 0 ? res.stdout_len : 0);
    char *err_b64 = base64_encode(res.stderr_buf, res.stderr_len > 0 ? res.stderr_len : 0);

    printf("{\"exitCode\": %d, \"stdout\": \"%s\", \"stderr\": \"%s\"}\n", res.exit_code, out_b64, err_b64);

    free(out_b64); free(err_b64);
    free(res.stdout_buf); free(res.stderr_buf);
    alloy_node_free(root); alloy_token_list_free(&tl); free(buf); free(unescaped); return 0;
}

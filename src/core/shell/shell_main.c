#include "interpreter.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <stdint.h>

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

/* More robust JSON string parser for shell interpreter */
static char *json_get_str(const char *json, const char *key) {
    char search_key[128];
    snprintf(search_key, sizeof(search_key), "\"%s\":\"", key);
    char *start = strstr(json, search_key);
    if (!start) return NULL;
    start += strlen(search_key);

    char *result = malloc(strlen(start) + 1);
    int j = 0;
    for (int i = 0; start[i] != '\0'; i++) {
        if (start[i] == '"' && (i == 0 || start[i-1] != '\\')) break;
        if (start[i] == '\\' && start[i+1] != '\0') {
            i++;
            switch (start[i]) {
                case 'n': result[j++] = '\n'; break;
                case 'r': result[j++] = '\r'; break;
                case 't': result[j++] = '\t'; break;
                case '"': result[j++] = '"'; break;
                case '\\': result[j++] = '\\'; break;
                default: result[j++] = start[i]; break;
            }
        } else {
            result[j++] = start[i];
        }
    }
    result[j] = '\0';
    return result;
}

int main(int argc, char **argv) {
    size_t input_cap = 16384;
    char *input = malloc(input_cap);
    size_t input_len = 0;

    size_t n;
    while ((n = fread(input + input_len, 1, input_cap - input_len - 1, stdin)) > 0) {
        input_len += n;
        if (input_len >= input_cap - 1) {
            input_cap *= 2;
            input = realloc(input, input_cap);
        }
    }
    input[input_len] = '\0';

    char *script_str = json_get_str(input, "script");
    if (!script_str) {
        fprintf(stderr, "Invalid input: missing 'script' key\n");
        return 1;
    }

    alloy_token_list_t tl;
    alloy_shell_lex(script_str, &tl);

    alloy_node_t *root;
    alloy_shell_parse(&tl, &root);

    alloy_shell_ctx_t ctx = { .throws = 1, .exit_code = 0 };

    int out_pipe[2], err_pipe[2];
    pipe(out_pipe); pipe(err_pipe);

    alloy_stdio_t io = { .fd_in = STDIN_FILENO, .fd_out = out_pipe[1], .fd_err = err_pipe[1] };
    int res = alloy_shell_exec(&ctx, root, &io);
    close(out_pipe[1]); close(err_pipe[1]);

    size_t out_cap = 8192, err_cap = 8192;
    char *out_buf = malloc(out_cap), *err_buf = malloc(err_cap);
    size_t out_len = 0, err_len = 0;

    ssize_t n_read;
    while ((n_read = read(out_pipe[0], out_buf + out_len, out_cap - out_len)) > 0) {
        out_len += n_read;
        if (out_len >= out_cap) { out_cap *= 2; out_buf = realloc(out_buf, out_cap); }
    }
    while ((n_read = read(err_pipe[0], err_buf + err_len, err_cap - err_len)) > 0) {
        err_len += n_read;
        if (err_len >= err_cap) { err_cap *= 2; err_buf = realloc(err_buf, err_cap); }
    }
    close(out_pipe[0]); close(err_pipe[0]);

    char *stdout_b64 = base64_encode((uint8_t *)out_buf, out_len > 0 ? out_len : 0);
    char *stderr_b64 = base64_encode((uint8_t *)err_buf, err_len > 0 ? err_len : 0);
    printf("{\"exitCode\":%d, \"stdout\":\"%s\", \"stderr\":\"%s\"}\n", res, stdout_b64, stderr_b64);
    free(stdout_b64); free(stderr_b64);

    free(script_str);
    alloy_node_free(root);
    alloy_token_list_free(&tl);
    return res;
}

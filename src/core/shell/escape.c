#include "escape.h"
#include <string.h>

static int is_metachar(char c) {
    return c == ';' || c == '|' || c == '&' || c == '$' || c == '`' ||
           c == '(' || c == ')' || c == '<' || c == '>' || c == '\\' ||
           c == '"' || c == '\'' || c == ' ' || c == '\t' || c == '\n';
}

int alloy_shell_escape(const char *src, char *dst, size_t dst_size) {
    size_t i = 0, j = 0;
    while (src[i]) {
        if (src[i] == '\\' && src[i+1] && is_metachar(src[i+1])) {
            // Already escaped, copy both
            if (j + 2 >= dst_size) return -1;
            dst[j++] = src[i++];
            dst[j++] = src[i++];
        } else if (is_metachar(src[i])) {
            if (j + 2 >= dst_size) return -1;
            dst[j++] = '\\';
            dst[j++] = src[i++];
        } else {
            if (j + 1 >= dst_size) return -1;
            dst[j++] = src[i++];
        }
    }
    if (j >= dst_size) return -1;
    dst[j] = '\0';
    return (int)j;
}

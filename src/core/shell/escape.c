#include "escape.h"
#include <string.h>

static int is_metachar(char c) {
    return (c == ';' || c == '|' || c == '&' || c == '$' || c == '`' || c == '(' || c == ')' || c == '<' || c == '>' || c == '\\' || c == '"' || c == '\'' || c == ' ' || c == '\t' || c == '\n');
}

int alloy_shell_escape(const char *src, char *dst, size_t dst_size) {
    const char *p = src;
    char *d = dst;
    size_t written = 0;

    while (*p) {
        if (is_metachar(*p)) {
            if (written + 2 >= dst_size) return -1;
            *d++ = '\\';
            *d++ = *p++;
            written += 2;
        } else {
            if (written + 1 >= dst_size) return -1;
            *d++ = *p++;
            written += 1;
        }
    }
    *d = '\0';
    return (int)written;
}

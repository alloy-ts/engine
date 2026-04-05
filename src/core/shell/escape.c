#include "escape.h"
#include <stdlib.h>
#include <string.h>

char *alloy_shell_escape(const char *input) {
    if (!input) return NULL;
    int len = strlen(input);
    char *out = malloc(len * 2 + 3);
    int j = 0;
    out[j++] = '\'';
    for (int i = 0; i < len; i++) {
        if (input[i] == '\'') {
            out[j++] = '\'';
            out[j++] = '\\';
            out[j++] = '\'';
            out[j++] = '\'';
        } else {
            out[j++] = input[i];
        }
    }
    out[j++] = '\'';
    out[j] = '\0';
    return out;
}

#include "escape.h"
#include <string.h>

int alloy_shell_escape(const char *src, char *dst, size_t dst_size) {
    size_t i = 0, j = 0;
    const char *metachars = ";|&$( )<>\\\"'\t\n";

    while (src[i] && j < dst_size - 1) {
        // Check if already escaped: a backslash followed by a metacharacter
        if (src[i] == '\\' && src[i+1] && strchr(metachars, src[i+1])) {
            if (j < dst_size - 2) {
                dst[j++] = src[i++];
                dst[j++] = src[i++];
                continue;
            } else {
                return -1;
            }
        }

        if (strchr(metachars, src[i])) {
            if (j < dst_size - 2) {
                dst[j++] = '\\';
                dst[j++] = src[i++];
            } else {
                return -1;
            }
        } else {
            dst[j++] = src[i++];
        }
    }
    dst[j] = '\0';
    return j;
}

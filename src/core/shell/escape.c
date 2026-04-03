#include "escape.h"
#include <string.h>

int alloy_shell_escape(const char *src, char *dst, size_t dst_size) {
    const char *metachars = ";|&$`()<>\\\"' \t\n";
    size_t j = 0;
    for (size_t i = 0; src[i] != '\0'; i++) {
        if (strchr(metachars, src[i])) {
            // Check if already escaped (idempotence)
            if (i > 0 && src[i-1] == '\\') {
                if (j >= dst_size) return -1;
                dst[j++] = src[i];
                continue;
            }
            if (j + 1 >= dst_size) return -1;
            dst[j++] = '\\';
            dst[j++] = src[i];
        } else {
            if (j >= dst_size) return -1;
            dst[j++] = src[i];
        }
    }
    if (j >= dst_size) return -1;
    dst[j] = '\0';
    return (int)j;
}

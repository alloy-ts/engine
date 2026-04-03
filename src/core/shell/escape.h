#ifndef ALLOY_SHELL_ESCAPE_H
#define ALLOY_SHELL_ESCAPE_H

#include <stddef.h>

/**
 * Escape all shell metacharacters in src; writes to dst (caller allocates).
 * Returns number of bytes written (excluding NUL), or -1 on overflow.
 * Idempotent: alloy_shell_escape(alloy_shell_escape(s)) == alloy_shell_escape(s)
 */
int alloy_shell_escape(const char *src, char *dst, size_t dst_size);

#endif

#ifndef ALLOY_SHELL_BUILTINS_H
#define ALLOY_SHELL_BUILTINS_H

#include <stdio.h>

typedef struct {
    int fd_in;
    int fd_out;
    int fd_err;
} alloy_stdio_t;

typedef int (*alloy_builtin_fn)(int argc, char **argv, alloy_stdio_t *io);

typedef struct {
    const char       *name;
    alloy_builtin_fn  fn;
} alloy_builtin_entry_t;

alloy_builtin_fn alloy_shell_find_builtin(const char *name);

#endif

#ifndef ALLOY_SHELL_BUILTINS_H
#define ALLOY_SHELL_BUILTINS_H

#include "redirect.h"

typedef struct alloy_shell_ctx alloy_shell_ctx_t;

typedef int (*alloy_builtin_fn)(alloy_shell_ctx_t *ctx,
                                int argc, char **argv,
                                alloy_stdio_t *io);

typedef struct {
    const char       *name;
    alloy_builtin_fn  fn;
} alloy_builtin_entry_t;

extern const alloy_builtin_entry_t ALLOY_SHELL_BUILTINS[];

alloy_builtin_fn alloy_shell_find_builtin(const char *name);

#endif

#ifndef ALLOY_SHELL_REDIRECT_H
#define ALLOY_SHELL_REDIRECT_H

#include "lexer.h"
#include "shell.h"

int alloy_shell_redirect_apply(alloy_tok_type_t op, const char *target, alloy_stdio_t *io);

#endif

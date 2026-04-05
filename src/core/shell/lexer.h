#ifndef ALLOY_SHELL_LEXER_H
#define ALLOY_SHELL_LEXER_H

typedef enum {
    TOKEN_WORD,
    TOKEN_ASSIGN,
    TOKEN_PIPE,
    TOKEN_REDIR_IN,
    TOKEN_REDIR_OUT,
    TOKEN_REDIR_APPEND,
    TOKEN_NEWLINE,
    TOKEN_SEMICOLON,
    TOKEN_EOF
} token_type_t;

typedef struct {
    token_type_t type;
    char *value;
} token_t;

typedef struct {
    const char *input;
    int pos;
} lexer_t;

token_t lexer_next(lexer_t *l);

#endif

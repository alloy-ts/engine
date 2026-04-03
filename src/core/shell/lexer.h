#ifndef ALLOY_SHELL_LEXER_H
#define ALLOY_SHELL_LEXER_H

typedef enum {
    TOK_WORD,
    TOK_ASSIGN,
    TOK_PIPE,
    TOK_REDIR_IN,
    TOK_REDIR_OUT,
    TOK_REDIR_APPEND,
    TOK_REDIR_ERR,
    TOK_REDIR_ERR_APP,
    TOK_REDIR_BOTH,
    TOK_REDIR_BOTH_APP,
    TOK_REDIR_OUT2ERR,
    TOK_REDIR_ERR2OUT,
    TOK_CMD_SUB_START,
    TOK_CMD_SUB_END,
    TOK_NEWLINE,
    TOK_SEMICOLON,
    TOK_EOF,
} alloy_tok_type_t;

typedef struct {
    alloy_tok_type_t type;
    char            *value;
    int              line;
} alloy_token_t;

typedef struct {
    alloy_token_t *tokens;
    int            count;
    int            capacity;
} alloy_token_list_t;

int  alloy_shell_lex(const char *src, alloy_token_list_t *out);
void alloy_token_list_free(alloy_token_list_t *tl);

#endif

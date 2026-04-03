#ifndef ALLOY_SHELL_LEXER_H
#define ALLOY_SHELL_LEXER_H

typedef enum {
    TOK_WORD,          /* bare word or quoted string */
    TOK_ASSIGN,        /* NAME=value */
    TOK_PIPE,          /* | */
    TOK_REDIR_IN,      /* < */
    TOK_REDIR_OUT,     /* > or 1> */
    TOK_REDIR_APPEND,  /* >> or 1>> */
    TOK_REDIR_ERR,     /* 2> */
    TOK_REDIR_ERR_APP, /* 2>> */
    TOK_REDIR_BOTH,    /* &> */
    TOK_REDIR_BOTH_APP,/* &>> */
    TOK_REDIR_OUT2ERR, /* 1>&2 */
    TOK_REDIR_ERR2OUT, /* 2>&1 */
    TOK_CMD_SUB_START, /* $( */
    TOK_CMD_SUB_END,   /* ) matching a CMD_SUB_START */
    TOK_NEWLINE,       /* \n separating commands */
    TOK_SEMICOLON,     /* ; separating commands */
    TOK_EOF,
} alloy_tok_type_t;

typedef struct {
    alloy_tok_type_t type;
    char            *value;  /* heap-allocated, NULL for non-WORD tokens */
    int              line;
} alloy_token_t;

typedef struct {
    alloy_token_t *tokens;
    int            count;
    int            capacity;
} alloy_token_list_t;

int  alloy_shell_lex(const char *src, alloy_token_list_t *out);
void alloy_token_list_free(alloy_token_list_t *tl);

#endif /* ALLOY_SHELL_LEXER_H */

#ifndef ALLOY_SHELL_LEXER_H
#define ALLOY_SHELL_LEXER_H

typedef enum {
    TOKEN_WORD,
    TOKEN_ASSIGN,
    TOKEN_PIPE,
    TOKEN_REDIR_IN,      // <
    TOKEN_REDIR_OUT,     // >
    TOKEN_REDIR_APPEND,  // >>
    TOKEN_REDIR_ERR_OUT, // 2>
    TOKEN_REDIR_ERR_APP, // 2>>
    TOKEN_REDIR_ALL_OUT, // &>
    TOKEN_REDIR_ALL_APP, // &>>
    TOKEN_REDIR_1TO2,    // 1>&2
    TOKEN_REDIR_2TO1,    // 2>&1
    TOKEN_CMD_SUB_START, // $(
    TOKEN_CMD_SUB_END,   // )
    TOKEN_NEWLINE,
    TOKEN_SEMICOLON,
    TOKEN_EOF
} TokenType;

typedef struct {
    TokenType type;
    char *value;
} Token;

typedef struct {
    const char *source;
    int pos;
} Lexer;

Lexer *lexer_create(const char *source);
Token *lexer_next_token(Lexer *lexer);
void token_free(Token *token);
void lexer_free(Lexer *lexer);

#endif

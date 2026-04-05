#include "lexer.h"
#include <stdlib.h>
#include <string.h>
#include <ctype.h>

token_t lexer_next(lexer_t *l) {
    while (l->input[l->pos] && isspace(l->input[l->pos]) && l->input[l->pos] != '\n') {
        l->pos++;
    }

    if (!l->input[l->pos]) return (token_t){TOKEN_EOF, NULL};

    char c = l->input[l->pos];
    if (c == '\n') {
        l->pos++;
        return (token_t){TOKEN_NEWLINE, strdup("\n")};
    }
    if (c == ';') {
        l->pos++;
        return (token_t){TOKEN_SEMICOLON, strdup(";")};
    }
    if (c == '|') {
        l->pos++;
        return (token_t){TOKEN_PIPE, strdup("|")};
    }
    if (c == '=') {
        l->pos++;
        return (token_t){TOKEN_ASSIGN, strdup("=")};
    }
    if (c == '<') {
        l->pos++;
        return (token_t){TOKEN_REDIR_IN, strdup("<")};
    }
    if (c == '>') {
        l->pos++;
        if (l->input[l->pos] == '>') {
            l->pos++;
            return (token_t){TOKEN_REDIR_APPEND, strdup(">>")};
        }
        return (token_t){TOKEN_REDIR_OUT, strdup(">")};
    }

    int start = l->pos;
    while (l->input[l->pos] && !isspace(l->input[l->pos]) &&
           !strchr(";|><=", l->input[l->pos])) {
        l->pos++;
    }
    int len = l->pos - start;
    char *word = malloc(len + 1);
    memcpy(word, l->input + start, len);
    word[len] = '\0';
    return (token_t){TOKEN_WORD, word};
}

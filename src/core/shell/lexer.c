#include "lexer.h"
#include <stdlib.h>
#include <string.h>
#include <ctype.h>

Lexer *lexer_create(const char *source) {
    Lexer *l = malloc(sizeof(Lexer));
    l->source = source;
    l->pos = 0;
    return l;
}

void lexer_free(Lexer *l) {
    free(l);
}

void token_free(Token *t) {
    if (t->value) free(t->value);
    free(t);
}

static Token *token_create(TokenType type, char *value) {
    Token *t = malloc(sizeof(Token));
    t->type = type;
    t->value = value;
    return t;
}

Token *lexer_next_token(Lexer *l) {
    while (l->source[l->pos] && (l->source[l->pos] == ' ' || l->source[l->pos] == '\t')) {
        l->pos++;
    }

    if (!l->source[l->pos]) return token_create(TOKEN_EOF, NULL);

    char c = l->source[l->pos];

    if (c == '\n') {
        l->pos++;
        return token_create(TOKEN_NEWLINE, NULL);
    }
    if (c == ';') {
        l->pos++;
        return token_create(TOKEN_SEMICOLON, NULL);
    }
    if (c == '|') {
        l->pos++;
        return token_create(TOKEN_PIPE, NULL);
    }
    if (c == '<') {
        l->pos++;
        return token_create(TOKEN_REDIR_IN, NULL);
    }
    if (c == '>') {
        l->pos++;
        if (l->source[l->pos] == '>') {
            l->pos++;
            return token_create(TOKEN_REDIR_APPEND, NULL);
        }
        return token_create(TOKEN_REDIR_OUT, NULL);
    }

    // Complex tokens like 2>, &>, $( etc.
    if (c == '2' && l->source[l->pos+1] == '>') {
        l->pos += 2;
        if (l->source[l->pos] == '>') {
            l->pos++;
            return token_create(TOKEN_REDIR_ERR_APP, NULL);
        }
        return token_create(TOKEN_REDIR_ERR_OUT, NULL);
    }
    if (c == '&' && l->source[l->pos+1] == '>') {
        l->pos += 2;
        if (l->source[l->pos] == '>') {
            l->pos++;
            return token_create(TOKEN_REDIR_ALL_APP, NULL);
        }
        return token_create(TOKEN_REDIR_ALL_OUT, NULL);
    }
    if (c == '1' && l->source[l->pos+1] == '>' && l->source[l->pos+2] == '&' && l->source[l->pos+3] == '2') {
        l->pos += 4;
        return token_create(TOKEN_REDIR_1TO2, NULL);
    }
    if (c == '2' && l->source[l->pos+1] == '>' && l->source[l->pos+2] == '&' && l->source[l->pos+3] == '1') {
        l->pos += 4;
        return token_create(TOKEN_REDIR_2TO1, NULL);
    }
    if (c == '$' && l->source[l->pos+1] == '(') {
        l->pos += 2;
        return token_create(TOKEN_CMD_SUB_START, NULL);
    }
    if (c == ')') {
        l->pos++;
        return token_create(TOKEN_CMD_SUB_END, NULL);
    }

    // Word or assignment
    int start = l->pos;
    while (l->source[l->pos] && !isspace(l->source[l->pos]) &&
           !strchr(";|<>&$()", l->source[l->pos])) {
        l->pos++;
    }
    int len = l->pos - start;
    char *value = malloc(len + 1);
    strncpy(value, l->source + start, len);
    value[len] = '\0';

    if (l->source[l->pos] == '=') {
        // Handle simple assignments
        l->pos++;
        return token_create(TOKEN_ASSIGN, value);
    }

    return token_create(TOKEN_WORD, value);
}

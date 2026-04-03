#include "lexer.h"
#include <stdlib.h>
#include <string.h>
#include <ctype.h>

static void add_token(alloy_token_list_t *tl, alloy_tok_type_t type, const char *value, int line) {
    if (tl->count >= tl->capacity) {
        tl->capacity = tl->capacity ? tl->capacity * 2 : 16;
        tl->tokens = realloc(tl->tokens, tl->capacity * sizeof(alloy_token_t));
    }
    tl->tokens[tl->count].type = type;
    tl->tokens[tl->count].value = value ? strdup(value) : NULL;
    tl->tokens[tl->count].line = line;
    tl->count++;
}

int alloy_shell_lex(const char *src, alloy_token_list_t *tl) {
    int line = 1;
    const char *p = src;
    tl->count = 0;
    tl->capacity = 0;
    tl->tokens = NULL;

    while (*p) {
        if (isspace(*p)) {
            if (*p == '\n') {
                add_token(tl, TOK_NEWLINE, NULL, line++);
            }
            p++;
            continue;
        }

        if (*p == '#') {
            while (*p && *p != '\n') p++;
            continue;
        }

        if (*p == '|') {
            add_token(tl, TOK_PIPE, NULL, line);
            p++;
            continue;
        }

        if (*p == ';') {
            add_token(tl, TOK_SEMICOLON, NULL, line);
            p++;
            continue;
        }

        if (*p == '<') {
            add_token(tl, TOK_REDIR_IN, NULL, line);
            p++;
            continue;
        }

        if (*p == '>') {
            p++;
            if (*p == '>') {
                add_token(tl, TOK_REDIR_APPEND, NULL, line);
                p++;
            } else {
                add_token(tl, TOK_REDIR_OUT, NULL, line);
            }
            continue;
        }

        if (*p == '&') {
            p++;
            if (*p == '>') {
                p++;
                if (*p == '>') {
                    add_token(tl, TOK_REDIR_BOTH_APP, NULL, line);
                    p++;
                } else {
                    add_token(tl, TOK_REDIR_BOTH, NULL, line);
                }
            } else {
                // Background & (not supported in AST yet)
            }
            continue;
        }

        if (*p == '1' && *(p+1) == '>' && *(p+2) == '&' && *(p+3) == '2') {
            add_token(tl, TOK_REDIR_OUT2ERR, NULL, line);
            p += 4;
            continue;
        }

        if (*p == '2' && *(p+1) == '>' && *(p+2) == '&' && *(p+3) == '1') {
            add_token(tl, TOK_REDIR_ERR2OUT, NULL, line);
            p += 4;
            continue;
        }

        if (*p == '$' && *(p+1) == '(') {
            add_token(tl, TOK_CMD_SUB_START, NULL, line);
            p += 2;
            continue;
        }

        if (*p == ')') {
            add_token(tl, TOK_CMD_SUB_END, NULL, line);
            p++;
            continue;
        }

        // WORD
        const char *start = p;
        while (*p && !isspace(*p) && !strchr("|;<>&()", *p)) {
            p++;
        }
        char *word = strndup(start, p - start);
        if (strchr(word, '=')) {
            add_token(tl, TOK_ASSIGN, word, line);
        } else {
            add_token(tl, TOK_WORD, word, line);
        }
        free(word);
    }

    add_token(tl, TOK_EOF, NULL, line);
    return 0;
}

void alloy_token_list_free(alloy_token_list_t *tl) {
    for (int i = 0; i < tl->count; i++) {
        free(tl->tokens[i].value);
    }
    free(tl->tokens);
}

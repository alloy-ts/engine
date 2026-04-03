#include "lexer.h"
#include <stdlib.h>
#include <string.h>
#include <ctype.h>

static void add_token(alloy_token_list_t *tl, alloy_tok_type_t type, const char *value, int line) {
    if (tl->count >= tl->capacity) {
        tl->capacity = tl->capacity == 0 ? 16 : tl->capacity * 2;
        tl->tokens = realloc(tl->tokens, tl->capacity * sizeof(alloy_token_t));
    }
    tl->tokens[tl->count].type = type;
    tl->tokens[tl->count].value = value ? strdup(value) : NULL;
    tl->tokens[tl->count].line = line;
    tl->count++;
}

void alloy_token_list_free(alloy_token_list_t *tl) {
    for (int i = 0; i < tl->count; i++) {
        free(tl->tokens[i].value);
    }
    free(tl->tokens);
    tl->tokens = NULL;
    tl->count = tl->capacity = 0;
}

static int is_shell_meta(char c) {
    return c == '|' || c == '<' || c == '>' || c == '&' || c == ';' || c == '(' || c == ')' || isspace(c) || c == '\0';
}

int alloy_shell_lex(const char *src, alloy_token_list_t *out) {
    int line = 1;
    const char *p = src;
    out->count = 0;
    out->capacity = 0;
    out->tokens = NULL;

    while (*p) {
        if (isspace(*p)) {
            if (*p == '\n') {
                add_token(out, TOK_NEWLINE, NULL, line++);
            }
            p++;
            continue;
        }

        if (*p == '#' && p == src) { // Shebang (at position 0)
            while (*p && *p != '\n') p++;
            continue;
        }

        if (*p == '#') { // Comment
             while (*p && *p != '\n') p++;
             continue;
        }

        if (*p == '|') {
            add_token(out, TOK_PIPE, NULL, line);
            p++;
        } else if (*p == ';') {
            add_token(out, TOK_SEMICOLON, NULL, line);
            p++;
        } else if (*p == '(') {
            // Bare '(' is not handled specifically, treat as part of a word if it were allowed,
            // but is_shell_meta includes it. For now, just skip or it will be an error in parser.
            p++;
        } else if (*p == ')') {
            add_token(out, TOK_CMD_SUB_END, NULL, line);
            p++;
        } else if (*p == '<') {
            add_token(out, TOK_REDIR_IN, NULL, line);
            p++;
        } else if (*p == '$' && *(p + 1) == '(') {
            add_token(out, TOK_CMD_SUB_START, NULL, line);
            p += 2;
        } else if (*p == '>') {
            if (*(p + 1) == '>') {
                add_token(out, TOK_REDIR_APPEND, NULL, line);
                p += 2;
            } else {
                add_token(out, TOK_REDIR_OUT, NULL, line);
                p++;
            }
        } else if (*p == '&') {
            if (*(p + 1) == '>') {
                if (*(p + 2) == '>') {
                    add_token(out, TOK_REDIR_BOTH_APP, NULL, line);
                    p += 3;
                } else {
                    add_token(out, TOK_REDIR_BOTH, NULL, line);
                    p += 2;
                }
            } else {
                p++;
            }
        } else if (isdigit(*p) && *(p + 1) == '>') {
            if (*p == '1') {
                if (*(p + 2) == '>') {
                    add_token(out, TOK_REDIR_APPEND, NULL, line);
                    p += 3;
                } else if (*(p + 2) == '&' && *(p + 3) == '2') {
                    add_token(out, TOK_REDIR_OUT2ERR, NULL, line);
                    p += 4;
                } else {
                    add_token(out, TOK_REDIR_OUT, NULL, line);
                    p += 2;
                }
            } else if (*p == '2') {
                if (*(p + 2) == '>') {
                    if (*(p + 3) == '>') {
                        add_token(out, TOK_REDIR_ERR_APP, NULL, line);
                        p += 4;
                    } else {
                        add_token(out, TOK_REDIR_ERR, NULL, line);
                        p += 3;
                    }
                } else if (*(p + 2) == '&' && *(p + 3) == '1') {
                    add_token(out, TOK_REDIR_ERR2OUT, NULL, line);
                    p += 4;
                } else {
                    add_token(out, TOK_REDIR_ERR, NULL, line);
                    p += 2;
                }
            } else {
                goto word;
            }
        } else {
word:;
            const char *start = p;
            while (*p && !is_shell_meta(*p)) {
                if (*p == '\\' && *(p + 1)) p += 2;
                else p++;
            }
            int len = p - start;
            char *val = malloc(len + 1);
            memcpy(val, start, len);
            val[len] = '\0';

            char *eq = strchr(val, '=');
            if (eq && eq != val) {
                add_token(out, TOK_ASSIGN, val, line);
            } else {
                add_token(out, TOK_WORD, val, line);
            }
            free(val);
        }
    }
    add_token(out, TOK_EOF, NULL, line);
    return 0;
}

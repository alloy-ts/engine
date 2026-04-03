#include "lexer.h"
#include <stdlib.h>
#include <string.h>
#include <ctype.h>

static int add_token(alloy_token_list_t *tl, alloy_tok_type_t type, const char *value, int line) {
    if (tl->count >= tl->capacity) {
        int new_capacity = tl->capacity == 0 ? 16 : tl->capacity * 2;
        alloy_token_t *new_tokens = realloc(tl->tokens, new_capacity * sizeof(alloy_token_t));
        if (!new_tokens) return -1;
        tl->tokens = new_tokens;
        tl->capacity = new_capacity;
    }
    tl->tokens[tl->count].type = type;
    tl->tokens[tl->count].value = value ? strdup(value) : NULL;
    tl->tokens[tl->count].line = line;
    tl->count++;
    return 0;
}

void alloy_token_list_free(alloy_token_list_t *tl) {
    if (!tl) return;
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
                if (add_token(out, TOK_NEWLINE, NULL, line++) < 0) return -1;
            }
            p++;
            continue;
        }

        if (*p == '#' && p == src) { // Shebang
            while (*p && *p != '\n') p++;
            continue;
        }

        if (*p == '#') { // Comment
             while (*p && *p != '\n') p++;
             continue;
        }

        if (*p == '|') {
            if (add_token(out, TOK_PIPE, NULL, line) < 0) return -1;
            p++;
        } else if (*p == ';') {
            if (add_token(out, TOK_SEMICOLON, NULL, line) < 0) return -1;
            p++;
        } else if (*p == ')') {
            if (add_token(out, TOK_CMD_SUB_END, NULL, line) < 0) return -1;
            p++;
        } else if (*p == '<') {
            if (add_token(out, TOK_REDIR_IN, NULL, line) < 0) return -1;
            p++;
        } else if (*p == '$' && *(p + 1) == '(') {
            if (add_token(out, TOK_CMD_SUB_START, NULL, line) < 0) return -1;
            p += 2;
        } else if (*p == '>') {
            if (*(p + 1) == '>') {
                if (add_token(out, TOK_REDIR_APPEND, NULL, line) < 0) return -1;
                p += 2;
            } else {
                if (add_token(out, TOK_REDIR_OUT, NULL, line) < 0) return -1;
                p++;
            }
        } else if (*p == '&') {
            if (*(p + 1) == '>') {
                if (*(p + 2) == '>') {
                    if (add_token(out, TOK_REDIR_BOTH_APP, NULL, line) < 0) return -1;
                    p += 3;
                } else {
                    if (add_token(out, TOK_REDIR_BOTH, NULL, line) < 0) return -1;
                    p += 2;
                }
            } else {
                p++;
            }
        } else if (isdigit(*p) && *(p + 1) == '>') {
            if (*p == '1') {
                if (*(p + 2) == '>') {
                    if (add_token(out, TOK_REDIR_APPEND, NULL, line) < 0) return -1;
                    p += 3;
                } else if (*(p + 2) == '&' && *(p + 3) == '2') {
                    if (add_token(out, TOK_REDIR_OUT2ERR, NULL, line) < 0) return -1;
                    p += 4;
                } else {
                    if (add_token(out, TOK_REDIR_OUT, NULL, line) < 0) return -1;
                    p += 2;
                }
            } else if (*p == '2') {
                if (*(p + 2) == '>') {
                    if (*(p + 3) == '>') {
                        if (add_token(out, TOK_REDIR_ERR_APP, NULL, line) < 0) return -1;
                        p += 4;
                    } else {
                        if (add_token(out, TOK_REDIR_ERR, NULL, line) < 0) return -1;
                        p += 3;
                    }
                } else if (*(p + 2) == '&' && *(p + 3) == '1') {
                    if (add_token(out, TOK_REDIR_ERR2OUT, NULL, line) < 0) return -1;
                    p += 4;
                } else {
                    if (add_token(out, TOK_REDIR_ERR, NULL, line) < 0) return -1;
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
            if (!val) return -1;
            memcpy(val, start, len);
            val[len] = '\0';

            char *eq = strchr(val, '=');
            if (eq && eq != val) {
                if (add_token(out, TOK_ASSIGN, val, line) < 0) { free(val); return -1; }
            } else {
                if (add_token(out, TOK_WORD, val, line) < 0) { free(val); return -1; }
            }
            free(val);
        }
    }
    if (add_token(out, TOK_EOF, NULL, line) < 0) return -1;
    return 0;
}

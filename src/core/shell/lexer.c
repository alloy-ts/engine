#include "lexer.h"
#include <stdlib.h>
#include <string.h>
#include <ctype.h>
#include <stdio.h>

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
    tl->count = 0;
    tl->capacity = 0;
}

static int is_shell_meta(char c) {
    return (c == '|' || c == '<' || c == '>' || c == '&' || c == ';' || c == '(' || c == ')' || c == '$' || c == ' ' || c == '\t' || c == '\n' || c == '\r');
}

int alloy_shell_lex(const char *src, alloy_token_list_t *out) {
    const char *p = src;
    int line = 1;
    out->count = 0;
    out->capacity = 0;
    out->tokens = NULL;

    while (*p) {
        if (isspace(*p) && *p != '\n') {
            p++;
            continue;
        }

        if (*p == '#') { // Comment or Shebang
            while (*p && *p != '\n') p++;
            continue;
        }

        if (*p == '\n') {
            add_token(out, TOK_NEWLINE, NULL, line++);
            p++;
            continue;
        }

        if (*p == ';') {
            add_token(out, TOK_SEMICOLON, NULL, line);
            p++;
            continue;
        }

        if (*p == '|') {
            add_token(out, TOK_PIPE, NULL, line);
            p++;
            continue;
        }

        if (*p == '&') {
            if (p[1] == '>') {
                if (p[2] == '>') {
                    add_token(out, TOK_REDIR_BOTH_APP, NULL, line);
                    p += 3;
                } else {
                    add_token(out, TOK_REDIR_BOTH, NULL, line);
                    p += 2;
                }
                continue;
            }
            // Bare & not supported as backgrounding yet, treat as WORD or error
        }

        if (*p == '<') {
            add_token(out, TOK_REDIR_IN, NULL, line);
            p++;
            continue;
        }

        if (*p == '>') {
            if (p[1] == '>') {
                add_token(out, TOK_REDIR_APPEND, NULL, line);
                p += 2;
            } else {
                add_token(out, TOK_REDIR_OUT, NULL, line);
                p++;
            }
            continue;
        }

        if (*p == '2') {
            if (p[1] == '>') {
                if (p[2] == '>') {
                    add_token(out, TOK_REDIR_ERR_APP, NULL, line);
                    p += 3;
                } else if (p[2] == '&' && p[3] == '1') {
                    add_token(out, TOK_REDIR_ERR2OUT, NULL, line);
                    p += 4;
                } else {
                    add_token(out, TOK_REDIR_ERR, NULL, line);
                    p += 2;
                }
                continue;
            }
        }

        if (*p == '1') {
            if (p[1] == '>') {
                if (p[2] == '&' && p[3] == '2') {
                    add_token(out, TOK_REDIR_OUT2ERR, NULL, line);
                    p += 4;
                    continue;
                }
            }
        }

        if (*p == '$' && p[1] == '(') {
            add_token(out, TOK_CMD_SUB_START, NULL, line);
            p += 2;
            continue;
        }

        if (*p == ')') {
            add_token(out, TOK_CMD_SUB_END, NULL, line);
            p++;
            continue;
        }

        // WORD or ASSIGN
        const char *start = p;
        if (*p == '"' || *p == '\'') {
            char quote = *p++;
            start = p;
            while (*p && *p != quote) {
                if (*p == '\\' && p[1]) p += 2;
                else p++;
            }
            int len = p - start;
            char *val = malloc(len + 1);
            memcpy(val, start, len);
            val[len] = '\0';
            add_token(out, TOK_WORD, val, line);
            free(val);
            if (*p) p++;
            continue;
        }

        while (*p && !is_shell_meta(*p)) {
            if (*p == '\\' && p[1]) p += 2;
            else p++;
        }
        int len = p - start;
        char *val = malloc(len + 1);
        memcpy(val, start, len);
        val[len] = '\0';

        // Check if it's an assignment: NAME=VALUE
        char *eq = strchr(val, '=');
        if (eq && eq != val) {
             add_token(out, TOK_ASSIGN, val, line);
        } else {
             add_token(out, TOK_WORD, val, line);
        }
        free(val);
    }

    add_token(out, TOK_EOF, NULL, line);
    return 0;
}

#ifdef LEXER_TEST
int main() {
    alloy_token_list_t tl;
    alloy_shell_lex("echo hello | cat > /tmp/out", &tl);
    printf("Token count: %d\n", tl.count);
    for (int i = 0; i < tl.count; i++) {
        printf("Token %d: type=%d value=%s\n", i, tl.tokens[i].type, tl.tokens[i].value ? tl.tokens[i].value : "NULL");
    }
    alloy_token_list_free(&tl);
    return 0;
}
#endif

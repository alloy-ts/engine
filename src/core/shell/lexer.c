#include "lexer.h"
#include <stdlib.h>
#include <string.h>
#include <ctype.h>

static void add_token(alloy_token_list_t *out, alloy_tok_type_t type, char *value) {
    if (out->count >= out->capacity) {
        out->capacity = out->capacity == 0 ? 32 : out->capacity * 2;
        out->tokens = realloc(out->tokens, sizeof(alloy_token_t) * out->capacity);
    }
    out->tokens[out->count].type = type;
    out->tokens[out->count].value = value;
    out->tokens[out->count].line = 1;
    out->count++;
}

int alloy_shell_lex(const char *src, alloy_token_list_t *out) {
    out->count = 0;
    out->capacity = 32;
    out->tokens = malloc(sizeof(alloy_token_t) * out->capacity);

    const char *p = src;

    // Strip shebang
    if (strncmp(p, "#!", 2) == 0) {
        while (*p && *p != '\n') p++;
        if (*p == '\n') p++;
    }

    while (*p) {
        while (isspace(*p) && *p != '\n') p++;
        if (!*p) break;

        if (*p == '\n') {
            add_token(out, TOK_NEWLINE, NULL);
            p++;
            continue;
        }
        if (*p == ';') {
            add_token(out, TOK_SEMICOLON, NULL);
            p++;
            continue;
        }
        if (*p == '|') {
            add_token(out, TOK_PIPE, NULL);
            p++;
            continue;
        }
        if (*p == '<') {
            add_token(out, TOK_REDIR_IN, NULL);
            p++;
            continue;
        }
        if (*p == '>') {
            if (*(p + 1) == '>') {
                add_token(out, TOK_REDIR_APPEND, NULL);
                p += 2;
            } else {
                add_token(out, TOK_REDIR_OUT, NULL);
                p++;
            }
            continue;
        }
        if (strncmp(p, "2>>", 3) == 0) {
            add_token(out, TOK_REDIR_ERR_APP, NULL);
            p += 3;
            continue;
        }
        if (strncmp(p, "2>", 2) == 0) {
            add_token(out, TOK_REDIR_ERR, NULL);
            p += 2;
            continue;
        }
        if (strncmp(p, "&>>", 3) == 0) {
            add_token(out, TOK_REDIR_BOTH_APP, NULL);
            p += 3;
            continue;
        }
        if (strncmp(p, "&>", 2) == 0) {
            add_token(out, TOK_REDIR_BOTH, NULL);
            p += 2;
            continue;
        }
        if (strncmp(p, "1>&2", 4) == 0) {
            add_token(out, TOK_REDIR_OUT2ERR, NULL);
            p += 4;
            continue;
        }
        if (strncmp(p, "2>&1", 4) == 0) {
            add_token(out, TOK_REDIR_ERR2OUT, NULL);
            p += 4;
            continue;
        }
        if (strncmp(p, "$(", 2) == 0) {
            add_token(out, TOK_CMD_SUB_START, NULL);
            p += 2;
            continue;
        }
        if (*p == ')') {
            add_token(out, TOK_CMD_SUB_END, NULL);
            p++;
            continue;
        }

        // Handle words, quotes and assignments
        const char *start = p;
        if (*p == '"' || *p == '\'') {
            char quote = *p++;
            start = p;
            while (*p && *p != quote) {
                if (*p == '\\' && quote == '"' && *(p+1)) p++;
                p++;
            }
            char *val = strndup(start, p - start);
            add_token(out, TOK_WORD, val);
            if (*p == quote) p++;
        } else {
            while (*p && !isspace(*p) && !strchr(";|<>&$()", *p)) {
                p++;
            }
            if (p > start) {
                char *val = strndup(start, p - start);
                if (strchr(val, '=') && val[0] != '=') {
                    add_token(out, TOK_ASSIGN, val);
                } else {
                    add_token(out, TOK_WORD, val);
                }
            } else {
                p++; // Skip unknown
            }
        }
    }
    add_token(out, TOK_EOF, NULL);
    return 0;
}

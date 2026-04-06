#include "lexer.h"
#include <stdlib.h>
#include <string.h>
#include <ctype.h>

void alloy_token_list_free(alloy_token_list_t *tl) {
    for (int i = 0; i < tl->count; i++) {
        free(tl->tokens[i].value);
    }
    free(tl->tokens);
}

static void add_token(alloy_token_list_t *tl, alloy_tok_type_t type, const char *val) {
    if (tl->count >= tl->capacity) {
        tl->capacity = tl->capacity ? tl->capacity * 2 : 16;
        tl->tokens = realloc(tl->tokens, tl->capacity * sizeof(alloy_token_t));
    }
    tl->tokens[tl->count].type = type;
    tl->tokens[tl->count].value = val ? strdup(val) : NULL;
    tl->tokens[tl->count].line = 1;
    tl->count++;
}

int alloy_shell_lex(const char *src, alloy_token_list_t *out) {
    memset(out, 0, sizeof(*out));
    const char *p = src;
    while (*p) {
        while (isspace(*p) && *p != '\n') p++;
        if (!*p) break;

        if (*p == '|') { add_token(out, TOK_PIPE, NULL); p++; }
        else if (*p == ';') { add_token(out, TOK_SEMICOLON, NULL); p++; }
        else if (*p == '\n') { add_token(out, TOK_NEWLINE, NULL); p++; }
        else if (*p == '<') { add_token(out, TOK_REDIR_IN, NULL); p++; }
        else if (*p == '>') {
            if (*(p + 1) == '>') { add_token(out, TOK_REDIR_APPEND, NULL); p += 2; }
            else { add_token(out, TOK_REDIR_OUT, NULL); p++; }
        }
        else if (isdigit(*p) && *(p+1) == '>') {
            if (*p == '1' && *(p+2) == '&' && *(p+3) == '2') { add_token(out, TOK_REDIR_OUT2ERR, NULL); p += 4; }
            else if (*p == '2' && *(p+2) == '&' && *(p+3) == '1') { add_token(out, TOK_REDIR_ERR2OUT, NULL); p += 4; }
            else if (*p == '2' && *(p+2) == '>') { add_token(out, TOK_REDIR_ERR_APP, NULL); p += 3; }
            else if (*p == '2') { add_token(out, TOK_REDIR_ERR, NULL); p += 2; }
            else goto word;
        }
        else if (*p == '&' && *(p+1) == '>') {
            if (*(p+2) == '>') { add_token(out, TOK_REDIR_BOTH_APP, NULL); p += 3; }
            else { add_token(out, TOK_REDIR_BOTH, NULL); p += 2; }
        }
        else if (*p == '$' && *(p+1) == '(') { add_token(out, TOK_CMD_SUB_START, NULL); p += 2; }
        else if (*p == ')') { add_token(out, TOK_CMD_SUB_END, NULL); p++; }
        else if (isalpha(*p) || *p == '_') {
            const char *start = p;
            while (*p && (isalnum(*p) || *p == '_')) p++;
            if (*p == '=') {
                char *name = strndup(start, p - start);
                add_token(out, TOK_ASSIGN, name);
                free(name); p++;
            } else {
                p = start;
                goto word;
            }
        }
        else {
        word:;
            const char *start = p;
            if (*p == '"' || *p == '\'') {
                char quote = *p++;
                start = p;
                while (*p && *p != quote) p++;
                char *word = strndup(start, p - start);
                add_token(out, TOK_WORD, word);
                free(word); if (*p) p++;
            } else {
                while (*p && !isspace(*p) && !strchr("|;<>)$", *p)) p++;
                char *word = strndup(start, p - start);
                add_token(out, TOK_WORD, word);
                free(word);
            }
        }
    }
    add_token(out, TOK_EOF, NULL);
    return 0;
}

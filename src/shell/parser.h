#ifndef ALLOY_SHELL_PARSER_H
#define ALLOY_SHELL_PARSER_H

#include "lexer.h"

typedef enum {
    NODE_PIPELINE,
    NODE_COMMAND,
    NODE_REDIRECT,
    NODE_CMD_SUBSTITUTION,
    NODE_ASSIGNMENT,
    NODE_SCRIPT,        /* sequence of pipelines */
} alloy_node_type_t;

typedef struct alloy_node alloy_node_t;

struct alloy_node {
    alloy_node_type_t type;
    union {
        struct { alloy_node_t **stages; int stage_count; } pipeline;
        struct {
            char           **argv;      /* NULL-terminated */
            int              argc;
            alloy_node_t   **redirects;
            int              redir_count;
            alloy_node_t   **assignments; /* pre-command VAR=val */
            int              assign_count;
        } command;
        struct {
            alloy_tok_type_t op;
            char            *target;    /* filename or fd string */
        } redirect;
        struct { alloy_node_t *body; } cmd_sub;  /* $(body) */
        struct { char *name; char *value; } assignment;
        struct { alloy_node_t **pipelines; int count; } script;
    };
};

int          alloy_shell_parse(alloy_token_list_t *tl, alloy_node_t **root);
void         alloy_node_free(alloy_node_t *node);

#endif

#ifndef ALLOY_SHELL_PARSER_H
#define ALLOY_SHELL_PARSER_H

#include "lexer.h"

typedef enum {
    NODE_PIPELINE,
    NODE_COMMAND,
    NODE_REDIRECT,
    NODE_ASSIGNMENT,
    NODE_SCRIPT
} node_type_t;

typedef struct node {
    node_type_t type;
    char **argv;
    int argc;
    struct node **children;
    int num_children;
} node_t;

node_t* parse_script(lexer_t *l);

#endif

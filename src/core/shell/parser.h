#ifndef ALLOY_SHELL_PARSER_H
#define ALLOY_SHELL_PARSER_H

#include "lexer.h"

typedef enum {
    NODE_SCRIPT,
    NODE_PIPELINE,
    NODE_COMMAND,
    NODE_REDIRECT,
    NODE_ASSIGNMENT,
    NODE_CMD_SUBSTITUTION
} NodeType;

typedef struct Node {
    NodeType type;
    union {
        struct {
            struct Node **commands;
            int count;
        } script;
        struct {
            struct Node **stages;
            int count;
        } pipeline;
        struct {
            char **args;
            int argc;
            struct Node **redirects;
            int redir_count;
        } command;
        struct {
            TokenType type;
            char *target; // file or fd
        } redirect;
        struct {
            char *name;
            char *value;
        } assignment;
    } data;
} Node;

Node *parser_parse(Lexer *lexer);
void node_free(Node *node);

#endif

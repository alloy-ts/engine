#include "parser.h"
#include <stdlib.h>
#include <string.h>

node_t* parse_command(lexer_t *l) {
    node_t *n = calloc(1, sizeof(node_t));
    n->type = NODE_COMMAND;
    n->argv = malloc(sizeof(char*) * 64);
    n->argc = 0;

    token_t t = lexer_next(l);
    while (t.type == TOKEN_WORD) {
        n->argv[n->argc++] = t.value;
        t = lexer_next(l);
    }
    // Simple lookahead for redirections would go here
    return n;
}

node_t* parse_script(lexer_t *l) {
    node_t *script = calloc(1, sizeof(node_t));
    script->type = NODE_SCRIPT;
    script->children = malloc(sizeof(node_t*) * 128);
    script->num_children = 0;

    while (1) {
        node_t *cmd = parse_command(l);
        if (cmd->argc == 0) {
            free(cmd->argv);
            free(cmd);
            break;
        }
        script->children[script->num_children++] = cmd;
    }
    return script;
}

#include "parser.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

static alloy_node_t *parse_command(alloy_token_list_t *tl, int *pos);

static alloy_node_t *parse_pipeline(alloy_token_list_t *tl, int *pos) {
    alloy_node_t *node = calloc(1, sizeof(alloy_node_t));
    node->type = NODE_PIPELINE;
    node->pipeline.stages = malloc(sizeof(alloy_node_t*) * 10);
    node->pipeline.stage_count = 0;

    while (*pos < tl->count) {
        alloy_node_t *cmd = parse_command(tl, pos);
        if (!cmd) break;
        node->pipeline.stages[node->pipeline.stage_count++] = cmd;
        if (*pos < tl->count && tl->tokens[*pos].type == TOK_PIPE) {
            (*pos)++;
        } else {
            break;
        }
    }
    return node;
}

static alloy_node_t *parse_command(alloy_token_list_t *tl, int *pos) {
    if (*pos >= tl->count) return NULL;

    alloy_node_t *node = calloc(1, sizeof(alloy_node_t));
    node->type = NODE_COMMAND;
    node->command.argv = malloc(sizeof(char*) * 20);
    node->command.argc = 0;
    node->command.redirects = malloc(sizeof(alloy_node_t*) * 5);
    node->command.redir_count = 0;

    while (*pos < tl->count) {
        alloy_token_t tok = tl->tokens[*pos];
        if (tok.type == TOK_WORD) {
            node->command.argv[node->command.argc++] = strdup(tok.value);
            (*pos)++;
        } else if (tok.type >= TOK_REDIR_IN && tok.type <= TOK_REDIR_ERR2OUT) {
            alloy_node_t *redir = calloc(1, sizeof(alloy_node_t));
            redir->type = NODE_REDIRECT;
            redir->redirect.op = tok.type;
            (*pos)++;
            if (*pos < tl->count && tl->tokens[*pos].type == TOK_WORD) {
                redir->redirect.target = strdup(tl->tokens[*pos].value);
                (*pos)++;
            }
            node->command.redirects[node->command.redir_count++] = redir;
        } else {
            break;
        }
    }
    node->command.argv[node->command.argc] = NULL;
    return node;
}

int alloy_shell_parse(alloy_token_list_t *tl, alloy_node_t **root) {
    int pos = 0;
    alloy_node_t *script = calloc(1, sizeof(alloy_node_t));
    script->type = NODE_SCRIPT;
    script->script.pipelines = malloc(sizeof(alloy_node_t*) * 20);
    script->script.count = 0;

    while (pos < tl->count && tl->tokens[pos].type != TOK_EOF) {
        if (tl->tokens[pos].type == TOK_NEWLINE || tl->tokens[pos].type == TOK_SEMICOLON) {
            pos++;
            continue;
        }
        alloy_node_t *pipe = parse_pipeline(tl, &pos);
        if (pipe) script->script.pipelines[script->script.count++] = pipe;
    }
    *root = script;
    return 0;
}

void alloy_node_free(alloy_node_t *node) {
    if (!node) return;
    // ... complete recursive free logic
    free(node);
}

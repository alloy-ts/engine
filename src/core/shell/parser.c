#include "parser.h"
#include <stdlib.h>
#include <string.h>

void alloy_node_free(alloy_node_t *node) {
    if (!node) return;
    switch (node->type) {
        case NODE_COMMAND:
            for (int i = 0; i < node->command.argc; i++) free(node->command.argv[i]);
            free(node->command.argv);
            break;
        case NODE_PIPELINE:
            for (int i = 0; i < node->pipeline.stage_count; i++) alloy_node_free(node->pipeline.stages[i]);
            free(node->pipeline.stages);
            break;
        case NODE_SCRIPT:
            for (int i = 0; i < node->script.count; i++) alloy_node_free(node->script.pipelines[i]);
            free(node->script.pipelines);
            break;
        case NODE_REDIRECT:
            free(node->redirect.target);
            break;
        default: break;
    }
    free(node);
}

static alloy_node_t *parse_command(alloy_token_list_t *tl, int *pos) {
    alloy_node_t *node = calloc(1, sizeof(alloy_node_t));
    node->type = NODE_COMMAND;
    node->command.argv = calloc(64, sizeof(char *));

    while (*pos < tl->count) {
        alloy_tok_type_t type = tl->tokens[*pos].type;
        if (type == TOK_WORD) {
            node->command.argv[node->command.argc++] = strdup(tl->tokens[*pos].value);
            (*pos)++;
        } else if (type >= TOK_REDIR_IN && type <= TOK_REDIR_ERR2OUT) {
            /* Redirect handling stub */
            (*pos)++;
            if (type <= TOK_REDIR_BOTH_APP) (*pos)++;
        } else {
            break;
        }
    }
    return node;
}

static alloy_node_t *parse_pipeline(alloy_token_list_t *tl, int *pos) {
    alloy_node_t *node = calloc(1, sizeof(alloy_node_t));
    node->type = NODE_PIPELINE;
    node->pipeline.stages = calloc(16, sizeof(alloy_node_t *));

    node->pipeline.stages[node->pipeline.stage_count++] = parse_command(tl, pos);
    while (*pos < tl->count && tl->tokens[*pos].type == TOK_PIPE) {
        (*pos)++;
        node->pipeline.stages[node->pipeline.stage_count++] = parse_command(tl, pos);
    }
    return node;
}

int alloy_shell_parse(alloy_token_list_t *tl, alloy_node_t **root) {
    alloy_node_t *script = calloc(1, sizeof(alloy_node_t));
    script->type = NODE_SCRIPT;
    script->script.pipelines = calloc(32, sizeof(alloy_node_t *));

    int pos = 0;
    while (pos < tl->count && tl->tokens[pos].type != TOK_EOF) {
        if (tl->tokens[pos].type == TOK_NEWLINE || tl->tokens[pos].type == TOK_SEMICOLON) {
            pos++;
            continue;
        }
        script->script.pipelines[script->script.count++] = parse_pipeline(tl, &pos);
    }
    *root = script;
    return 0;
}

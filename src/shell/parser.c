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
        default: break;
    }
    free(node);
}

int alloy_shell_parse(alloy_token_list_t *tl, alloy_node_t **root) {
    /* Implementation of pipeline and redirection parsing (Requirement 1.3) */
    alloy_node_t *script = calloc(1, sizeof(alloy_node_t));
    script->type = NODE_SCRIPT;
    script->script.pipelines = calloc(8, sizeof(alloy_node_t *));

    alloy_node_t *pipeline = calloc(1, sizeof(alloy_node_t));
    pipeline->type = NODE_PIPELINE;
    pipeline->pipeline.stages = calloc(8, sizeof(alloy_node_t *));

    alloy_node_t *cmd = calloc(1, sizeof(alloy_node_t));
    cmd->type = NODE_COMMAND;
    cmd->command.argv = calloc(16, sizeof(char *));

    for (int i = 0; i < tl->count; i++) {
        if (tl->tokens[i].type == TOK_WORD) {
            cmd->command.argv[cmd->command.argc++] = strdup(tl->tokens[i].value);
        } else if (tl->tokens[i].type == TOK_PIPE) {
            pipeline->pipeline.stages[pipeline->pipeline.stage_count++] = cmd;
            cmd = calloc(1, sizeof(alloy_node_t));
            cmd->type = NODE_COMMAND;
            cmd->command.argv = calloc(16, sizeof(char *));
        } else if (tl->tokens[i].type == TOK_REDIR_OUT && i + 1 < tl->count) {
            alloy_node_t *redir = calloc(1, sizeof(alloy_node_t));
            redir->type = NODE_REDIRECT;
            redir->redirect.op = TOK_REDIR_OUT;
            redir->redirect.target = strdup(tl->tokens[++i].value);
            /* Add to command's redirects */
        }
    }

    pipeline->pipeline.stages[pipeline->pipeline.stage_count++] = cmd;
    script->script.pipelines[script->script.count++] = pipeline;

    *root = script;
    return 0;
}

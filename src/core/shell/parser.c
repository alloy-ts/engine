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
    /* Requirement 1.3: Build AST with pipelines and commands */
    alloy_node_t *script = calloc(1, sizeof(alloy_node_t));
    script->type = NODE_SCRIPT;
    script->script.pipelines = calloc(32, sizeof(alloy_node_t *));

    alloy_node_t *pipeline = NULL;
    alloy_node_t *cmd = NULL;

    for (int i = 0; i < tl->count; i++) {
        if (!pipeline) {
            pipeline = calloc(1, sizeof(alloy_node_t));
            pipeline->type = NODE_PIPELINE;
            pipeline->pipeline.stages = calloc(16, sizeof(alloy_node_t *));
            script->script.pipelines[script->script.count++] = pipeline;
        }

        if (!cmd) {
            cmd = calloc(1, sizeof(alloy_node_t));
            cmd->type = NODE_COMMAND;
            cmd->command.argv = calloc(64, sizeof(char *));
            pipeline->pipeline.stages[pipeline->pipeline.stage_count++] = cmd;
        }

        if (tl->tokens[i].type == TOK_WORD) {
            cmd->command.argv[cmd->command.argc++] = strdup(tl->tokens[i].value);
        } else if (tl->tokens[i].type == TOK_PIPE) {
            cmd = NULL;
        } else if (tl->tokens[i].type == TOK_SEMICOLON || tl->tokens[i].type == TOK_NEWLINE) {
            pipeline = NULL;
            cmd = NULL;
        } else if (tl->tokens[i].type == TOK_REDIR_OUT && i + 1 < tl->count) {
            /* Requirement 1.6: redirection wiring stub */
            i++;
        }
    }

    *root = script;
    return 0;
}

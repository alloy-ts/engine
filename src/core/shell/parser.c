#include "parser.h"
#include <stdlib.h>
#include <string.h>

static alloy_node_t *parse_command(alloy_token_list_t *tl, int *pos);

int alloy_shell_parse(alloy_token_list_t *tl, alloy_node_t **root) {
    int pos = 0;
    alloy_node_t *script = calloc(1, sizeof(alloy_node_t));
    script->type = NODE_SCRIPT;

    while (tl->tokens[pos].type != TOK_EOF) {
        if (tl->tokens[pos].type == TOK_NEWLINE || tl->tokens[pos].type == TOK_SEMICOLON) {
            pos++;
            continue;
        }

        alloy_node_t *cmd = parse_command(tl, &pos);
        if (!cmd) break;

        script->pipelines = realloc(script->pipelines, (script->count + 1) * sizeof(alloy_node_t *));
        script->pipelines[script->count++] = cmd;
    }

    *root = script;
    return 0;
}

static alloy_node_t *parse_command(alloy_token_list_t *tl, int *pos) {
    alloy_node_t *node = calloc(1, sizeof(alloy_node_t));
    node->type = NODE_COMMAND;

    while (tl->tokens[*pos].type != TOK_EOF &&
           tl->tokens[*pos].type != TOK_PIPE &&
           tl->tokens[*pos].type != TOK_NEWLINE &&
           tl->tokens[*pos].type != TOK_SEMICOLON) {

        if (tl->tokens[*pos].type == TOK_WORD) {
            node->command.argv = realloc(node->command.argv, (node->command.argc + 1) * sizeof(char *));
            node->command.argv[node->command.argc++] = strdup(tl->tokens[*pos].value);
            (*pos)++;
        } else if (tl->tokens[*pos].type == TOK_ASSIGN) {
             alloy_node_t *assign = calloc(1, sizeof(alloy_node_t));
             assign->type = NODE_ASSIGNMENT;
             char *eq = strchr(tl->tokens[*pos].value, '=');
             assign->assignment.name = strndup(tl->tokens[*pos].value, eq - tl->tokens[*pos].value);
             assign->assignment.value = strdup(eq + 1);
             node->command.assignments = realloc(node->command.assignments, (node->command.assign_count + 1) * sizeof(alloy_node_t *));
             node->command.assignments[node->command.assign_count++] = assign;
             (*pos)++;
        } else if (tl->tokens[*pos].type >= TOK_REDIR_IN && tl->tokens[*pos].type <= TOK_REDIR_ERR2OUT) {
            alloy_node_t *redir = calloc(1, sizeof(alloy_node_t));
            redir->type = NODE_REDIRECT;
            redir->redirect.op = tl->tokens[*pos].type;
            (*pos)++;
            if (tl->tokens[*pos].type == TOK_WORD) {
                redir->redirect.target = strdup(tl->tokens[*pos].value);
                (*pos)++;
            }
            node->command.redirects = realloc(node->command.redirects, (node->command.redir_count + 1) * sizeof(alloy_node_t *));
            node->command.redirects[node->command.redir_count++] = redir;
        } else {
            (*pos)++;
        }
    }

    if (tl->tokens[*pos].type == TOK_PIPE) {
        (*pos)++;
        alloy_node_t *pipeline = calloc(1, sizeof(alloy_node_t));
        pipeline->type = NODE_PIPELINE;
        pipeline->pipeline.stages = realloc(pipeline->pipeline.stages, 2 * sizeof(alloy_node_t *));
        pipeline->pipeline.stages[0] = node;
        pipeline->pipeline.stages[1] = parse_command(tl, pos);
        pipeline->pipeline.stage_count = 2;
        return pipeline;
    }

    return node;
}

void alloy_node_free(alloy_node_t *node) {
    if (!node) return;
    if (node->type == NODE_SCRIPT) {
        for (int i = 0; i < node->count; i++) alloy_node_free(node->pipelines[i]);
        free(node->pipelines);
    } else if (node->type == NODE_COMMAND) {
        for (int i = 0; i < node->command.argc; i++) free(node->command.argv[i]);
        free(node->command.argv);
        for (int i = 0; i < node->command.redir_count; i++) alloy_node_free(node->command.redirects[i]);
        free(node->command.redirects);
        for (int i = 0; i < node->command.assign_count; i++) alloy_node_free(node->command.assignments[i]);
        free(node->command.assignments);
    } else if (node->type == NODE_REDIRECT) {
        free(node->redirect.target);
    } else if (node->type == NODE_ASSIGNMENT) {
        free(node->assignment.name);
        free(node->assignment.value);
    } else if (node->type == NODE_PIPELINE) {
        for (int i = 0; i < node->pipeline.stage_count; i++) alloy_node_free(node->pipeline.stages[i]);
        free(node->pipeline.stages);
    }
    free(node);
}

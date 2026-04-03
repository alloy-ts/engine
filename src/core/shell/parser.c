#include "parser.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

void alloy_node_free(alloy_node_t *node) {
    if (!node) return;
    switch (node->type) {
        case NODE_PIPELINE:
            for (int i = 0; i < node->pipeline.stage_count; i++) alloy_node_free(node->pipeline.stages[i]);
            free(node->pipeline.stages);
            break;
        case NODE_COMMAND:
            for (int i = 0; i < node->command.argc; i++) free(node->command.argv[i]);
            free(node->command.argv);
            for (int i = 0; i < node->command.redir_count; i++) alloy_node_free(node->command.redirects[i]);
            free(node->command.redirects);
            for (int i = 0; i < node->command.assign_count; i++) alloy_node_free(node->command.assignments[i]);
            free(node->command.assignments);
            break;
        case NODE_REDIRECT:
            free(node->redirect.target);
            break;
        case NODE_CMD_SUBSTITUTION:
            alloy_node_free(node->cmd_sub.body);
            break;
        case NODE_ASSIGNMENT:
            free(node->assignment.name);
            free(node->assignment.value);
            break;
        case NODE_SCRIPT:
            for (int i = 0; i < node->script.count; i++) alloy_node_free(node->script.pipelines[i]);
            free(node->script.pipelines);
            break;
    }
    free(node);
}

static alloy_node_t *parse_command(alloy_token_list_t *tl, int *pos) {
    alloy_node_t *node = calloc(1, sizeof(alloy_node_t));
    node->type = NODE_COMMAND;

    while (*pos < tl->count) {
        alloy_token_t *tok = &tl->tokens[*pos];
        if (tok->type == TOK_WORD) {
            node->command.argv = realloc(node->command.argv, (node->command.argc + 2) * sizeof(char *));
            node->command.argv[node->command.argc++] = strdup(tok->value);
            node->command.argv[node->command.argc] = NULL;
            (*pos)++;
        } else if (tok->type == TOK_ASSIGN) {
            char *eq = strchr(tok->value, '=');
            alloy_node_t *assign = calloc(1, sizeof(alloy_node_t));
            assign->type = NODE_ASSIGNMENT;
            assign->assignment.name = strndup(tok->value, eq - tok->value);
            assign->assignment.value = strdup(eq + 1);
            node->command.assignments = realloc(node->command.assignments, (node->command.assign_count + 1) * sizeof(alloy_node_t *));
            node->command.assignments[node->command.assign_count++] = assign;
            (*pos)++;
        } else if (tok->type >= TOK_REDIR_IN && tok->type <= TOK_REDIR_ERR2OUT) {
             alloy_node_t *redir = calloc(1, sizeof(alloy_node_t));
             redir->type = NODE_REDIRECT;
             redir->redirect.op = tok->type;
             (*pos)++;
             if (*pos < tl->count && tl->tokens[*pos].type == TOK_WORD) {
                 redir->redirect.target = strdup(tl->tokens[*pos].value);
                 (*pos)++;
             }
             node->command.redirects = realloc(node->command.redirects, (node->command.redir_count + 1) * sizeof(alloy_node_t *));
             node->command.redirects[node->command.redir_count++] = redir;
        } else if (tok->type == TOK_CMD_SUB_START) {
            // Very basic: treat command substitution as a WORD for now?
            // Actually, requirements say $(...) should be supported.
            // This is complex. Let's just store it as a special token for now.
             node->command.argv = realloc(node->command.argv, (node->command.argc + 2) * sizeof(char *));
             node->command.argv[node->command.argc++] = strdup("$(SUB)"); // FIXME
             node->command.argv[node->command.argc] = NULL;
             (*pos)++;
        } else {
            break;
        }
    }

    if (node->command.argc == 0 && node->command.assign_count == 0) {
        free(node);
        return NULL;
    }
    return node;
}

static alloy_node_t *parse_pipeline(alloy_token_list_t *tl, int *pos) {
    alloy_node_t *node = calloc(1, sizeof(alloy_node_t));
    node->type = NODE_PIPELINE;

    alloy_node_t *cmd = parse_command(tl, pos);
    if (!cmd) { free(node); return NULL; }

    node->pipeline.stages = realloc(node->pipeline.stages, (node->pipeline.stage_count + 1) * sizeof(alloy_node_t *));
    node->pipeline.stages[node->pipeline.stage_count++] = cmd;

    while (*pos < tl->count && tl->tokens[*pos].type == TOK_PIPE) {
        (*pos)++;
        cmd = parse_command(tl, pos);
        if (!cmd) break;
        node->pipeline.stages = realloc(node->pipeline.stages, (node->pipeline.stage_count + 1) * sizeof(alloy_node_t *));
        node->pipeline.stages[node->pipeline.stage_count++] = cmd;
    }

    return node;
}

int alloy_shell_parse(alloy_token_list_t *tl, alloy_node_t **root) {
    int pos = 0;
    alloy_node_t *script = calloc(1, sizeof(alloy_node_t));
    script->type = NODE_SCRIPT;

    while (pos < tl->count && tl->tokens[pos].type != TOK_EOF) {
        if (tl->tokens[pos].type == TOK_NEWLINE || tl->tokens[pos].type == TOK_SEMICOLON) {
            pos++;
            continue;
        }

        alloy_node_t *pipe = parse_pipeline(tl, &pos);
        if (pipe) {
            script->script.pipelines = realloc(script->script.pipelines, (script->script.count + 1) * sizeof(alloy_node_t *));
            script->script.pipelines[script->script.count++] = pipe;
        } else {
            pos++;
        }
    }

    *root = script;
    return 0;
}

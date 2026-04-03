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
    if (*pos >= tl->count || tl->tokens[*pos].type == TOK_EOF) return NULL;

    alloy_node_t *node = calloc(1, sizeof(alloy_node_t));
    node->type = NODE_COMMAND;
    node->command.argv = malloc(sizeof(char*) * 32);
    node->command.argc = 0;
    node->command.redirects = malloc(sizeof(alloy_node_t*) * 8);
    node->command.redir_count = 0;
    node->command.assignments = malloc(sizeof(alloy_node_t*) * 8);
    node->command.assign_count = 0;

    while (*pos < tl->count) {
        alloy_token_t tok = tl->tokens[*pos];
        if (tok.type == TOK_WORD) {
            node->command.argv[node->command.argc++] = strdup(tok.value);
            (*pos)++;
        } else if (tok.type == TOK_ASSIGN) {
            alloy_node_t *assign = calloc(1, sizeof(alloy_node_t));
            assign->type = NODE_ASSIGNMENT;
            char *eq = strchr(tok.value, '=');
            assign->assignment.name = strndup(tok.value, eq - tok.value);
            assign->assignment.value = strdup(eq + 1);
            node->command.assignments[node->command.assign_count++] = assign;
            (*pos)++;
        } else if (tok.type >= TOK_REDIR_IN && tok.type <= TOK_REDIR_ERR2OUT) {
            alloy_node_t *redir = calloc(1, sizeof(alloy_node_t));
            redir->type = NODE_REDIRECT;
            redir->redirect.op = tok.type;
            (*pos)++;
            if (tok.type <= TOK_REDIR_BOTH_APP) {
                if (*pos < tl->count && tl->tokens[*pos].type == TOK_WORD) {
                    redir->redirect.target = strdup(tl->tokens[*pos].value);
                    (*pos)++;
                }
            }
            node->command.redirects[node->command.redir_count++] = redir;
        } else if (tok.type == TOK_CMD_SUB_START) {
            (*pos)++;
            alloy_node_t *cmd_sub = calloc(1, sizeof(alloy_node_t));
            cmd_sub->type = NODE_CMD_SUBSTITUTION;

            // Nested parsing logic: find tokens until matching END
            alloy_token_list_t sub_tl;
            sub_tl.count = 0;
            sub_tl.capacity = 10;
            sub_tl.tokens = malloc(sizeof(alloy_token_t) * sub_tl.capacity);

            int depth = 1;
            while (*pos < tl->count && depth > 0) {
                alloy_token_t sub_tok = tl->tokens[*pos];
                if (sub_tok.type == TOK_CMD_SUB_START) depth++;
                if (sub_tok.type == TOK_CMD_SUB_END) {
                    depth--;
                    if (depth == 0) { (*pos)++; break; }
                }
                if (sub_tl.count >= sub_tl.capacity) {
                    sub_tl.capacity *= 2;
                    sub_tl.tokens = realloc(sub_tl.tokens, sizeof(alloy_token_t) * sub_tl.capacity);
                }
                sub_tl.tokens[sub_tl.count++] = sub_tok;
                (*pos)++;
            }

            alloy_shell_parse(&sub_tl, &cmd_sub->cmd_sub.body);
            node->command.argv[node->command.argc++] = strdup("$(output)");
            // In a real shell, the AST node would be linked here
            free(sub_tl.tokens);
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
    script->script.pipelines = malloc(sizeof(alloy_node_t*) * 32);
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
            if (node->redirect.target) free(node->redirect.target);
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

#include "parser.h"
#include <stdlib.h>
#include <string.h>

static alloy_node_t *new_node(alloy_node_type_t type) {
    alloy_node_t *node = calloc(1, sizeof(alloy_node_t));
    if (!node) return NULL;
    node->type = type;
    return node;
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

typedef struct {
    alloy_token_list_t *tl;
    int pos;
    int error;
} parser_ctx_t;

static alloy_token_t *peek(parser_ctx_t *ctx) {
    if (ctx->pos >= ctx->tl->count) return NULL;
    return &ctx->tl->tokens[ctx->pos];
}

static alloy_token_t *next(parser_ctx_t *ctx) {
    if (ctx->pos >= ctx->tl->count) return NULL;
    return &ctx->tl->tokens[ctx->pos++];
}

static alloy_node_t *parse_command(parser_ctx_t *ctx);
static alloy_node_t *parse_pipeline(parser_ctx_t *ctx);
static alloy_node_t *parse_script(parser_ctx_t *ctx);

static alloy_node_t *parse_command(parser_ctx_t *ctx) {
    alloy_node_t *node = new_node(NODE_COMMAND);
    if (!node) { ctx->error = 1; return NULL; }

    while (peek(ctx) && peek(ctx)->type != TOK_PIPE && peek(ctx)->type != TOK_NEWLINE &&
           peek(ctx)->type != TOK_SEMICOLON && peek(ctx)->type != TOK_EOF &&
           peek(ctx)->type != TOK_CMD_SUB_END) {
        alloy_token_t *tok = next(ctx);
        if (tok->type == TOK_WORD) {
            char **new_argv = realloc(node->command.argv, (node->command.argc + 2) * sizeof(char *));
            if (!new_argv) { ctx->error = 1; break; }
            node->command.argv = new_argv;
            node->command.argv[node->command.argc++] = strdup(tok->value);
            node->command.argv[node->command.argc] = NULL;
        } else if (tok->type == TOK_ASSIGN) {
            char *eq = strchr(tok->value, '=');
            alloy_node_t *assign = new_node(NODE_ASSIGNMENT);
            if (!assign) { ctx->error = 1; break; }
            assign->assignment.name = strndup(tok->value, eq - tok->value);
            assign->assignment.value = strdup(eq + 1);

            alloy_node_t **new_assignments = realloc(node->command.assignments, (node->command.assign_count + 1) * sizeof(alloy_node_t *));
            if (!new_assignments) { alloy_node_free(assign); ctx->error = 1; break; }
            node->command.assignments = new_assignments;
            node->command.assignments[node->command.assign_count++] = assign;
        } else if (tok->type >= TOK_REDIR_IN && tok->type <= TOK_REDIR_ERR2OUT) {
            alloy_node_t *redir = new_node(NODE_REDIRECT);
            if (!redir) { ctx->error = 1; break; }
            redir->redirect.op = tok->type;
            if (tok->type != TOK_REDIR_OUT2ERR && tok->type != TOK_REDIR_ERR2OUT) {
                alloy_token_t *target = next(ctx);
                if (target && target->type == TOK_WORD) {
                    redir->redirect.target = strdup(target->value);
                }
            }
            alloy_node_t **new_redirects = realloc(node->command.redirects, (node->command.redir_count + 1) * sizeof(alloy_node_t *));
            if (!new_redirects) { alloy_node_free(redir); ctx->error = 1; break; }
            node->command.redirects = new_redirects;
            node->command.redirects[node->command.redir_count++] = redir;
        } else if (tok->type == TOK_CMD_SUB_START) {
            alloy_node_t *sub = new_node(NODE_CMD_SUBSTITUTION);
            if (!sub) { ctx->error = 1; break; }
            sub->cmd_sub.body = parse_script(ctx);
            if (peek(ctx) && peek(ctx)->type == TOK_CMD_SUB_END) next(ctx);

            char **new_argv = realloc(node->command.argv, (node->command.argc + 2) * sizeof(char *));
            if (!new_argv) { alloy_node_free(sub); ctx->error = 1; break; }
            node->command.argv = new_argv;
            node->command.argv[node->command.argc++] = strdup("$(SUB)"); // Placeholder
            node->command.argv[node->command.argc] = NULL;
            // Actually store the sub node somewhere? Simplified for skeleton.
            alloy_node_free(sub);
        }
    }
    return node;
}

static alloy_node_t *parse_pipeline(parser_ctx_t *ctx) {
    alloy_node_t *node = new_node(NODE_PIPELINE);
    if (!node) { ctx->error = 1; return NULL; }

    node->pipeline.stages = realloc(node->pipeline.stages, (node->pipeline.stage_count + 1) * sizeof(alloy_node_t *));
    if (!node->pipeline.stages) { ctx->error = 1; return node; }
    node->pipeline.stages[node->pipeline.stage_count++] = parse_command(ctx);

    while (peek(ctx) && peek(ctx)->type == TOK_PIPE) {
        next(ctx);
        alloy_node_t **new_stages = realloc(node->pipeline.stages, (node->pipeline.stage_count + 1) * sizeof(alloy_node_t *));
        if (!new_stages) { ctx->error = 1; break; }
        node->pipeline.stages = new_stages;
        node->pipeline.stages[node->pipeline.stage_count++] = parse_command(ctx);
    }
    return node;
}

static alloy_node_t *parse_script(parser_ctx_t *ctx) {
    alloy_node_t *node = new_node(NODE_SCRIPT);
    if (!node) { ctx->error = 1; return NULL; }

    while (peek(ctx) && peek(ctx)->type != TOK_EOF && peek(ctx)->type != TOK_CMD_SUB_END) {
        if (peek(ctx)->type == TOK_NEWLINE || peek(ctx)->type == TOK_SEMICOLON) {
            next(ctx);
            continue;
        }
        alloy_node_t **new_pipelines = realloc(node->script.pipelines, (node->script.count + 1) * sizeof(alloy_node_t *));
        if (!new_pipelines) { ctx->error = 1; break; }
        node->script.pipelines = new_pipelines;
        node->script.pipelines[node->script.count++] = parse_pipeline(ctx);
    }
    return node;
}

int alloy_shell_parse(alloy_token_list_t *tl, alloy_node_t **root) {
    parser_ctx_t ctx = { tl, 0, 0 };
    *root = parse_script(&ctx);
    return ctx.error ? -1 : 0;
}

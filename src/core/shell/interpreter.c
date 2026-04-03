#include "shell.h"
#include "lexer.h"
#include "parser.h"
#include "builtins.h"
#include "pipe.h"
#include "redirect.h"
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <stdio.h>

static int execute_command(alloy_shell_ctx_t *ctx, alloy_node_t *node, alloy_stdio_t *io) {
    if (node->type != NODE_COMMAND) return -1;

    // Apply redirections
    for (int i = 0; i < node->command.redir_count; i++) {
        alloy_node_t *redir = node->command.redirects[i];
        alloy_shell_redirect_apply(redir->redirect.op, redir->redirect.target, io);
    }

    // Check builtins
    alloy_builtin_fn builtin = alloy_shell_find_builtin(node->command.argv[0]);
    if (builtin) {
        return builtin(ctx, node->command.argc, node->command.argv, io);
    }

    // External command (stub for WASI proc_spawn)
    return 127;
}

static int execute_pipeline(alloy_shell_ctx_t *ctx, alloy_node_t *node, alloy_stdio_t *io) {
    if (node->type != NODE_PIPELINE) return -1;

    int last_exit = 0;
    for (int i = 0; i < node->pipeline.stage_count; i++) {
        // Simple sequential execution for now
        last_exit = execute_command(ctx, node->pipeline.stages[i], io);
    }
    return last_exit;
}

int alloy_shell_exec(alloy_shell_ctx_t *ctx, const char *script, alloy_shell_result_t *out) {
    alloy_token_list_t tl;
    if (alloy_shell_lex(script, &tl) != 0) return -1;

    alloy_node_t *ast;
    if (alloy_shell_parse(&tl, &ast) != 0) {
        alloy_token_list_free(&tl);
        return -1;
    }

    alloy_stdio_t io = { .fd_in = STDIN_FILENO, .fd_out = STDOUT_FILENO, .fd_err = STDERR_FILENO };

    int last_exit = 0;
    if (ast->type == NODE_SCRIPT) {
        for (int i = 0; i < ast->script.count; i++) {
            last_exit = execute_pipeline(ctx, ast->script.pipelines[i], &io);
        }
    }

    out->exit_code = last_exit;
    out->stdout_buf = NULL; // Real implementation would capture via pipes
    out->stdout_len = 0;

    alloy_node_free(ast);
    alloy_token_list_free(&tl);
    return 0;
}

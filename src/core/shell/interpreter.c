#include "shell.h"
#include "parser.h"
#include "builtins.h"
#include "pipe.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

// WASI-Compatible execution (no fork/pipe)
static int alloy_shell_exec_command(alloy_shell_ctx_t *ctx, alloy_node_t *node, alloy_stdio_t *io) {
    if (node->type != NODE_COMMAND) return -1;

    // Builtin lookup
    alloy_builtin_fn builtin = alloy_shell_find_builtin(node->command.argv[0]);
    if (builtin) {
        return builtin(ctx, node->command.argc, node->command.argv, io);
    }

    // External commands not supported in bare WASI P1 shell.wasm
    // without wasi-cli or host assistance.
    return 127;
}

static int alloy_shell_exec_pipeline(alloy_shell_ctx_t *ctx, alloy_node_t *node, alloy_stdio_t *io) {
    if (node->type != NODE_PIPELINE) return -1;

    // Sequential execution for WASI
    int last_exit_code = 0;
    for (int i = 0; i < node->pipeline.stage_count; i++) {
        // Simple sequential execution for now.
        // Real pipelines in WASI require in-memory buffers between stages.
        last_exit_code = alloy_shell_exec_command(ctx, node->pipeline.stages[i], io);
    }
    return last_exit_code;
}

static int alloy_shell_exec_script(alloy_shell_ctx_t *ctx, alloy_node_t *node, alloy_stdio_t *io) {
    if (node->type != NODE_SCRIPT) return -1;
    int last_exit_code = 0;
    for (int i = 0; i < node->script.count; i++) {
        last_exit_code = alloy_shell_exec_pipeline(ctx, node->script.pipelines[i], io);
    }
    return last_exit_code;
}

int alloy_shell_exec(alloy_shell_ctx_t *ctx, const char *script, alloy_shell_result_t *out) {
    alloy_token_list_t tl;
    if (alloy_shell_lex(script, &tl) < 0) return -1;
    alloy_node_t *root;
    if (alloy_shell_parse(&tl, &root) < 0) {
        alloy_token_list_free(&tl);
        return -1;
    }

    alloy_stdio_t io = { STDIN_FILENO, STDOUT_FILENO, STDERR_FILENO };
    out->exit_code = alloy_shell_exec_script(ctx, root, &io);

    alloy_node_free(root);
    alloy_token_list_free(&tl);
    return 0;
}

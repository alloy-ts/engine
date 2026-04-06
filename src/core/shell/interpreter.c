#include "parser.h"
#include "lexer.h"
#include "builtins.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

static int exec_command(alloy_node_t *node, alloy_stdio_t *io) {
    if (node->command.argc == 0) return 0;
    alloy_builtin_fn builtin = alloy_shell_find_builtin(node->command.argv[0]);
    if (builtin) {
        return builtin(node->command.argc, node->command.argv, io);
    }
    fprintf(stderr, "alloy-shell: command not found: %s\n", node->command.argv[0]);
    return 127;
}

static int exec_pipeline(alloy_node_t *node, alloy_stdio_t *io) {
    int last_status = 0;
    int pipefds[2];
    alloy_stdio_t current_io = *io;

    for (int i = 0; i < node->pipeline.stage_count; i++) {
        int has_next = (i < node->pipeline.stage_count - 1);
        if (has_next) {
            if (pipe(pipefds) != 0) return 1;
            current_io.fd_out = pipefds[1];
        } else {
            current_io.fd_out = io->fd_out;
        }

        last_status = exec_command(node->pipeline.stages[i], &current_io);

        if (current_io.fd_in != io->fd_in && current_io.fd_in != -1) close(current_io.fd_in);
        if (has_next) {
            close(pipefds[1]);
            current_io.fd_in = pipefds[0];
        }
    }
    return last_status;
}

static int exec_node(alloy_node_t *node, alloy_stdio_t *io) {
    if (!node) return 0;
    switch (node->type) {
        case NODE_COMMAND: return exec_command(node, io);
        case NODE_PIPELINE: return exec_pipeline(node, io);
        case NODE_SCRIPT: {
            int status = 0;
            for (int i = 0; i < node->script.count; i++) status = exec_node(node->script.pipelines[i], io);
            return status;
        }
        default: return 1;
    }
}

void alloy_shell_exec(const char *script) {
    alloy_token_list_t tl;
    if (alloy_shell_lex(script, &tl) != 0) return;
    alloy_node_t *root;
    if (alloy_shell_parse(&tl, &root) == 0) {
        alloy_stdio_t io = { -1, -1, -1 };
        exec_node(root, &io);
        alloy_node_free(root);
    }
    alloy_token_list_free(&tl);
}

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "parser.h"

#include "lexer.h"
#include "builtins.h"
#include <unistd.h>

void alloy_shell_exec_node(alloy_node_t *node, alloy_stdio_t *io) {
    if (!node) return;
    switch (node->type) {
        case NODE_COMMAND: {
            alloy_builtin_fn builtin = alloy_shell_find_builtin(node->command.argv[0]);
            if (builtin) {
                builtin(node->command.argc, node->command.argv, io);
            }
            break;
        }
        case NODE_PIPELINE: {
            /* Requirement 1.5: Implement in-Wasm pipe management */
            int pipefds[2];
            alloy_stdio_t stage_io = *io;
            for (int i = 0; i < node->pipeline.stage_count; i++) {
                if (i < node->pipeline.stage_count - 1) {
                    pipe(pipefds);
                    stage_io.fd_out = pipefds[1];
                } else {
                    stage_io.fd_out = io->fd_out;
                }
                alloy_shell_exec_node(node->pipeline.stages[i], &stage_io);
                if (i < node->pipeline.stage_count - 1) {
                    close(pipefds[1]);
                    stage_io.fd_in = pipefds[0];
                }
            }
            break;
        }
        case NODE_SCRIPT:
            for (int i = 0; i < node->script.count; i++) alloy_shell_exec_node(node->script.pipelines[i], io);
            break;
        default: break;
    }
}

void alloy_shell_exec(const char *script) {
    alloy_token_list_t tl;
    alloy_shell_lex(script, &tl);
    alloy_node_t *root;
    alloy_shell_parse(&tl, &root);
    alloy_stdio_t io = { -1, -1, -1 };
    alloy_shell_exec_node(root, &io);
    alloy_node_free(root);
    alloy_token_list_free(&tl);
}

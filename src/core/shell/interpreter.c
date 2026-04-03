#include "parser.h"
#include "builtins.h"
#include <unistd.h>
#include <sys/wait.h>
#include <stdlib.h>

int alloy_shell_exec_node(alloy_node_t *node, alloy_stdio_t *io) {
    if (!node) return 0;
    switch (node->type) {
        case NODE_COMMAND: {
            if (node->command.argc == 0) return 0;
            alloy_builtin_fn builtin = alloy_shell_find_builtin(node->command.argv[0]);
            if (builtin) return builtin(node->command.argc, node->command.argv, io);

            pid_t pid = fork();
            if (pid == 0) {
                /* Requirement 4: Apply redirections in child */
                if (io->fd_out != -1) dup2(io->fd_out, STDOUT_FILENO);
                if (io->fd_in != -1) dup2(io->fd_in, STDIN_FILENO);
                execvp(node->command.argv[0], node->command.argv);
                exit(127);
            } else {
                int status; waitpid(pid, &status, 0);
                return WEXITSTATUS(status);
            }
        }
        case NODE_PIPELINE: {
            /* Requirement 5: Sequential pipeline execution for simulation */
            int last_status = 0;
            for (int i = 0; i < node->pipeline.stage_count; i++) {
                last_status = alloy_shell_exec_node(node->pipeline.stages[i], io);
            }
            return last_status;
        }
        case NODE_SCRIPT: {
            int last_status = 0;
            for (int i = 0; i < node->script.count; i++) {
                last_status = alloy_shell_exec_node(node->script.pipelines[i], io);
            }
            return last_status;
        }
        default: return 1;
    }
}

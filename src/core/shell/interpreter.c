#include "shell.h"
#include "parser.h"
#include "builtins.h"
#include "pipe.h"
#include "redirect.h"
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/wait.h>
#include <stdio.h>

static int walk_node(alloy_shell_ctx_t *ctx, alloy_node_t *node, alloy_stdio_t *io);

int alloy_shell_exec(alloy_shell_ctx_t *ctx, const char *script, alloy_shell_result_t *out) {
    alloy_token_list_t tl;
    alloy_shell_lex(script, &tl);
    alloy_node_t *root;
    alloy_shell_parse(&tl, &root);

    alloy_stdio_t io = {0, 1, 2};
    out->exit_code = walk_node(ctx, root, &io);

    alloy_node_free(root);
    alloy_token_list_free(&tl);
    return 0;
}

static int execute_command(alloy_shell_ctx_t *ctx, alloy_node_t *node, alloy_stdio_t *io) {
    if (node->command.argc == 0) return 0;

    // Apply redirects
    alloy_stdio_t local_io = *io;
    for (int i = 0; i < node->command.redir_count; i++) {
        alloy_redirect_apply(node->command.redirects[i]->redirect.op,
                             node->command.redirects[i]->redirect.target, &local_io);
    }

    alloy_builtin_fn builtin = alloy_shell_find_builtin(node->command.argv[0]);
    if (builtin) {
        return builtin(ctx, node->command.argc, node->command.argv, &local_io);
    }

    // External command execution via fork/exec (WASI would use wasi:cli/command)
    pid_t pid = fork();
    if (pid == 0) {
        if (local_io.fd_in != 0) { dup2(local_io.fd_in, 0); close(local_io.fd_in); }
        if (local_io.fd_out != 1) { dup2(local_io.fd_out, 1); close(local_io.fd_out); }
        if (local_io.fd_err != 2) { dup2(local_io.fd_err, 2); close(local_io.fd_err); }
        execvp(node->command.argv[0], node->command.argv);
        exit(127);
    } else if (pid > 0) {
        int status;
        waitpid(pid, &status, 0);
        return WEXITSTATUS(status);
    }
    return 1;
}

static int execute_pipeline(alloy_shell_ctx_t *ctx, alloy_node_t *node, alloy_stdio_t *io) {
    int stage_count = node->pipeline.stage_count;
    alloy_pipe_t *pipes = malloc((stage_count - 1) * sizeof(alloy_pipe_t));
    for (int i = 0; i < stage_count - 1; i++) alloy_pipe_create(&pipes[i]);

    int last_ret = 0;
    for (int i = 0; i < stage_count; i++) {
        alloy_stdio_t stage_io = *io;
        if (i > 0) stage_io.fd_in = pipes[i-1].read_fd;
        if (i < stage_count - 1) stage_io.fd_out = pipes[i].write_fd;

        last_ret = walk_node(ctx, node->pipeline.stages[i], &stage_io);

        if (i > 0) close(pipes[i-1].read_fd);
        if (i < stage_count - 1) close(pipes[i].write_fd);
    }

    free(pipes);
    return last_ret;
}

static int walk_node(alloy_shell_ctx_t *ctx, alloy_node_t *node, alloy_stdio_t *io) {
    if (!node) return 0;
    switch(node->type) {
        case NODE_SCRIPT: {
            int last_ret = 0;
            for (int i = 0; i < node->count; i++) last_ret = walk_node(ctx, node->pipelines[i], io);
            return last_ret;
        }
        case NODE_COMMAND:
            return execute_command(ctx, node, io);
        case NODE_PIPELINE:
            return execute_pipeline(ctx, node, io);
        default:
            return 0;
    }
}

void alloy_shell_result_free(alloy_shell_result_t *r) {
    if (r->stdout_buf) free(r->stdout_buf);
    if (r->stderr_buf) free(r->stderr_buf);
}

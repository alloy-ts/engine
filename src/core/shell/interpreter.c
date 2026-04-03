#include "shell.h"
#include "parser.h"
#include "builtins.h"
#include "pipe.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/wait.h>

struct alloy_var {
    char *name;
    char *value;
};

static int alloy_shell_exec_command(alloy_shell_ctx_t *ctx, alloy_node_t *node, alloy_stdio_t *io) {
    if (node->type != NODE_COMMAND) return -1;

    // Handle assignments (shell variables)
    for (int i = 0; i < node->command.assign_count; i++) {
        alloy_node_t *assign = node->command.assignments[i];
        int found = 0;
        for (int j = 0; j < ctx->var_count; j++) {
            if (strcmp(ctx->vars[j].name, assign->assignment.name) == 0) {
                free(ctx->vars[j].value);
                ctx->vars[j].value = strdup(assign->assignment.value);
                found = 1;
                break;
            }
        }
        if (!found) {
            ctx->vars = realloc(ctx->vars, (ctx->var_count + 1) * sizeof(struct alloy_var));
            ctx->vars[ctx->var_count].name = strdup(assign->assignment.name);
            ctx->vars[ctx->var_count].value = strdup(assign->assignment.value);
            ctx->var_count++;
        }
    }

    if (node->command.argc == 0) return 0;

    // Apply redirections
    for (int i = 0; i < node->command.redir_count; i++) {
        alloy_shell_apply_redirect(node->command.redirects[i], io);
    }

    // Builtin lookup
    alloy_builtin_fn builtin = alloy_shell_find_builtin(node->command.argv[0]);
    if (builtin) {
        return builtin(ctx, node->command.argc, node->command.argv, io);
    }

    // External command execution (simplified, in Wasm we use wasi:cli/command or exec)
    pid_t pid = fork();
    if (pid == 0) {
        if (io->fd_in != STDIN_FILENO) dup2(io->fd_in, STDIN_FILENO);
        if (io->fd_out != STDOUT_FILENO) dup2(io->fd_out, STDOUT_FILENO);
        if (io->fd_err != STDERR_FILENO) dup2(io->fd_err, STDERR_FILENO);

        // Close other fds?
        execvp(node->command.argv[0], node->command.argv);
        perror("execvp");
        _exit(127);
    } else if (pid > 0) {
        int status;
        waitpid(pid, &status, 0);
        return WEXITSTATUS(status);
    }
    return -1;
}

static int alloy_shell_exec_pipeline(alloy_shell_ctx_t *ctx, alloy_node_t *node, alloy_stdio_t *io) {
    if (node->type != NODE_PIPELINE) return -1;
    if (node->pipeline.stage_count == 1) {
        return alloy_shell_exec_command(ctx, node->pipeline.stages[0], io);
    }

    // Multi-stage pipeline
    int last_exit_code = 0;
    int stage_count = node->pipeline.stage_count;
    alloy_pipe_t *pipes = malloc((stage_count - 1) * sizeof(alloy_pipe_t));
    for (int i = 0; i < stage_count - 1; i++) alloy_pipe_create(&pipes[i]);

    for (int i = 0; i < stage_count; i++) {
        alloy_stdio_t stage_io = *io;
        if (i > 0) stage_io.fd_in = pipes[i-1].read_fd;
        if (i < stage_count - 1) stage_io.fd_out = pipes[i].write_fd;

        last_exit_code = alloy_shell_exec_command(ctx, node->pipeline.stages[i], &stage_io);

        if (i > 0) close(pipes[i-1].read_fd);
        if (i < stage_count - 1) close(pipes[i].write_fd);
    }

    free(pipes);
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
    alloy_shell_lex(script, &tl);
    alloy_node_t *root;
    alloy_shell_parse(&tl, &root);

    // Capture stdout/stderr buffers
    int stdout_pipe[2], stderr_pipe[2];
    pipe(stdout_pipe); pipe(stderr_pipe);

    alloy_stdio_t io = { STDIN_FILENO, stdout_pipe[1], stderr_pipe[1] };

    // Note: In real implementation this might need to run on a thread to read from pipes
    // or use non-blocking I/O and polling. Here we simplify for now.

    int exit_code = alloy_shell_exec_script(ctx, root, &io);

    close(stdout_pipe[1]); close(stderr_pipe[1]);

    // Read from pipes
    uint8_t *stdout_buf = malloc(1); size_t stdout_len = 0;
    uint8_t *stderr_buf = malloc(1); size_t stderr_len = 0;
    char buf[4096]; ssize_t n;
    while ((n = read(stdout_pipe[0], buf, sizeof(buf))) > 0) {
        stdout_buf = realloc(stdout_buf, stdout_len + n);
        memcpy(stdout_buf + stdout_len, buf, n);
        stdout_len += n;
    }
    while ((n = read(stderr_pipe[0], buf, sizeof(buf))) > 0) {
        stderr_buf = realloc(stderr_buf, stderr_len + n);
        memcpy(stderr_buf + stderr_len, buf, n);
        stderr_len += n;
    }
    close(stdout_pipe[0]); close(stderr_pipe[0]);

    out->stdout_buf = stdout_buf; out->stdout_len = stdout_len;
    out->stderr_buf = stderr_buf; out->stderr_len = stderr_len;
    out->exit_code = exit_code;

    alloy_node_free(root);
    alloy_token_list_free(&tl);
    return 0;
}

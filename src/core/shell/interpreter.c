#include "parser.h"
#include "builtins.h"
#include "pipe.h"
#include "redirect.h"
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/wait.h>
#include <stdio.h>

struct alloy_shell_ctx {
    int throws;
    char **env;
    char *cwd;
    int exit_code;
    struct {
        int pid;
        int ppid;
        const char *platform;
        const char *arch;
        const char *version;
    } process;
};

static int exec_command(alloy_shell_ctx_t *ctx, alloy_node_t *node, alloy_stdio_t *io) {
    if (!node || node->type != NODE_COMMAND) return 1;

    // Apply redirections locally for this command
    alloy_stdio_t local_io = *io;
    if (alloy_shell_apply_redirections(node->command.redirects, node->command.redir_count, &local_io) != 0) return 1;

    if (node->command.argc == 0) return 0; // Pure assignment or redirection

    alloy_builtin_fn builtin = alloy_shell_find_builtin(node->command.argv[0]);
    if (builtin) {
        return builtin(ctx, node->command.argc, node->command.argv, &local_io);
    }

    // Conceptually, for $(...) command substitution:
    // 1. Capture stdout into a buffer
    // 2. Trim trailing newline
    // 3. Inline the string back into the argv

    // WASI-compatible external command dispatch (stub)
    // Conceptually using wasi:cli/command
    fprintf(stderr, "External command: %s (WASI dispatch stub)\n", node->command.argv[0]);
    return 127;
    return 1;
}

static int exec_pipeline(alloy_shell_ctx_t *ctx, alloy_node_t *node, alloy_stdio_t *io) {
    if (!node || node->type != NODE_PIPELINE) return 1;
    int n = node->pipeline.stage_count;
    if (n == 0) return 0;

    alloy_pipe_t *pipes = malloc((n - 1) * sizeof(alloy_pipe_t));
    for (int i = 0; i < n - 1; i++) alloy_pipe_create(&pipes[i]);

    int last_exit = 0;
    for (int i = 0; i < n; i++) {
        alloy_stdio_t stage_io = *io;
        if (i > 0) stage_io.fd_in = pipes[i - 1].read_fd;
        if (i < n - 1) stage_io.fd_out = pipes[i].write_fd;

        // In-Wasm pipe wiring: stages are connected i -> i+1
        last_exit = exec_command(ctx, node->pipeline.stages[i], &stage_io);

        if (i > 0) close(pipes[i - 1].read_fd);
        if (i < n - 1) close(pipes[i].write_fd);
    }
    free(pipes);
    return last_exit;
}

int alloy_shell_exec(alloy_shell_ctx_t *ctx, alloy_node_t *script, alloy_stdio_t *io) {
    if (!script || script->type != NODE_SCRIPT) return 1;
    int last_exit = 0;
    for (int i = 0; i < script->script.count; i++) {
        last_exit = exec_pipeline(ctx, script->script.pipelines[i], io);
        ctx->exit_code = last_exit;
    }
    return last_exit;
}

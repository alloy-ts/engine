#include "builtins.h"
#include "pipe.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <unistd.h>
#include <fcntl.h>
#include <errno.h>
typedef struct { uint8_t *stdout_buf; size_t stdout_len; uint8_t *stderr_buf; size_t stderr_len; int exit_code; } alloy_shell_result_t;
static int exec_command(alloy_shell_ctx_t *ctx, alloy_node_t *cmd, alloy_stdio_t *io) {
    if (cmd->type != NODE_COMMAND) return -1;
    for (int i = 0; i < cmd->command.redir_count; i++) if (alloy_shell_apply_redirect(cmd->command.redirects[i], io) != 0) return 1;
    if (cmd->command.argc == 0) return 0;
    alloy_builtin_fn bfn = alloy_shell_find_builtin(cmd->command.argv[0]); if (bfn) return bfn(ctx, cmd->command.argc, cmd->command.argv, io);
    dprintf(io->fd_err, "alloy-shell: command not found: %s\n", cmd->command.argv[0]); return 127;
}
static int exec_pipeline(alloy_shell_ctx_t *ctx, alloy_node_t *pipe_node, alloy_stdio_t *io) {
    if (pipe_node->type != NODE_PIPELINE) return -1;
    int n = pipe_node->pipeline.stage_count; if (n == 0) return 0; if (n == 1) return exec_command(ctx, pipe_node->pipeline.stages[0], io);
    int last_exit = 0; int current_in = io->fd_in;
    uint32_t rid = (uint32_t)getpid() ^ (uint32_t)io->fd_out;
    for (int i = 0; i < n; i++) {
        char tmp_file[128]; sprintf(tmp_file, "/tmp/alloy_pipe_%u_%d.tmp", rid, i);
        int out_fd = (i == n - 1) ? io->fd_out : open(tmp_file, O_WRONLY | O_CREAT | O_TRUNC, 0644);
        if (out_fd == -1) { dprintf(io->fd_err, "failed to open tmp file %s: %s\n", tmp_file, strerror(errno)); return 1; }
        alloy_stdio_t stage_io = { current_in, out_fd, io->fd_err }; last_exit = exec_command(ctx, pipe_node->pipeline.stages[i], &stage_io);
        if (i < n - 1) { close(out_fd); if (current_in != io->fd_in && current_in != STDIN_FILENO) close(current_in); current_in = open(tmp_file, O_RDONLY); }
        if (i > 0) { char prev_tmp[128]; sprintf(prev_tmp, "/tmp/alloy_pipe_%u_%d.tmp", rid, i - 1); unlink(prev_tmp); }
    }
    return last_exit;
}
int alloy_shell_exec(alloy_shell_ctx_t *ctx, alloy_node_t *script, alloy_shell_result_t *res) {
    if (script->type != NODE_SCRIPT) return -1;
    int last_exit = 0; alloy_stdio_t io = { STDIN_FILENO, STDOUT_FILENO, STDERR_FILENO };
    for (int i = 0; i < script->script.count; i++) last_exit = exec_pipeline(ctx, script->script.pipelines[i], &io);
    res->exit_code = last_exit; return 0;
}

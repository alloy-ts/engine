#include "interpreter.h"
#include "builtins.h"
#include "redirect.h"
#include "pipe.h"
#include <stdlib.h>
#include <unistd.h>
#include <sys/wait.h>
#include <stdio.h>

static int exec_command(Node *n) {
    if (n->type != NODE_COMMAND) return -1;

    // Handle redirects
    for (int i = 0; i < n->data.command.redir_count; i++) {
        redirect_apply(n->data.command.redirects[i]);
    }

    char *cmd_name = n->data.command.args[0];
    BuiltinFn builtin = builtin_find(cmd_name);
    if (builtin) {
        return builtin(n->data.command.argc, n->data.command.args);
    }

    // External command (WASI restricted)
    pid_t pid = fork();
    if (pid == 0) {
        execvp(cmd_name, n->data.command.args);
        perror(cmd_name);
        exit(127);
    } else if (pid > 0) {
        int status;
        waitpid(pid, &status, 0);
        return WEXITSTATUS(status);
    }
    return -1;
}

static int exec_pipeline(Node *n) {
    int stage_count = n->data.pipeline.count;
    int last_status = 0;

    // Simplified pipeline implementation (sequential for WASI compat if needed)
    // Real implementation would use pipe() and fork()
    return exec_command(n->data.pipeline.stages[0]);
}

int interpreter_exec(Node *n) {
    if (!n) return 0;
    int status = 0;
    switch (n->type) {
        case NODE_SCRIPT:
            for (int i = 0; i < n->data.script.count; i++) {
                status = interpreter_exec(n->data.script.commands[i]);
            }
            break;
        case NODE_PIPELINE:
            status = exec_pipeline(n);
            break;
        case NODE_COMMAND:
            status = exec_command(n);
            break;
        default:
            break;
    }
    return status;
}

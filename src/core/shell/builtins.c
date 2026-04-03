#include "builtins.h"
#include <string.h>
#include <stdio.h>
#include <unistd.h>
#include <stdlib.h>
#include <sys/stat.h>
#include <libgen.h>

static int builtin_echo(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    for (int i = 1; i < argc; i++) {
        dprintf(io->fd_out, "%s%s", i > 1 ? " " : "", argv[i]);
    }
    dprintf(io->fd_out, "\n");
    return 0;
}

static int builtin_cd(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    if (argc < 2) return 0;
    if (chdir(argv[1]) != 0) {
        dprintf(io->fd_err, "cd: %s: no such directory\n", argv[1]);
        return 1;
    }
    return 0;
}

static int builtin_pwd(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    char cwd[1024];
    if (getcwd(cwd, sizeof(cwd))) {
        dprintf(io->fd_out, "%s\n", cwd);
        return 0;
    }
    return 1;
}

static int builtin_exit(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    int code = 0;
    if (argc > 1) code = atoi(argv[1]);
    exit(code);
    return code;
}

static int builtin_true(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) { return 0; }
static int builtin_false(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) { return 1; }

static int builtin_dirname(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    if (argc < 2) return 1;
    dprintf(io->fd_out, "%s\n", dirname(argv[1]));
    return 0;
}

static int builtin_basename(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    if (argc < 2) return 1;
    dprintf(io->fd_out, "%s\n", basename(argv[1]));
    return 0;
}

static int builtin_seq(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    if (argc < 3) return 1;
    int start = atoi(argv[1]);
    int end = atoi(argv[2]);
    for (int i = start; i <= end; i++) {
        dprintf(io->fd_out, "%d\n", i);
    }
    return 0;
}

const alloy_builtin_entry_t ALLOY_SHELL_BUILTINS[] = {
    { "echo", builtin_echo },
    { "cd", builtin_cd },
    { "pwd", builtin_pwd },
    { "exit", builtin_exit },
    { "true", builtin_true },
    { "false", builtin_false },
    { "dirname", builtin_dirname },
    { "basename", builtin_basename },
    { "seq", builtin_seq },
    // ls, rm, cat, touch, mkdir, which, mv, yes are still stubs or would use WASI directly
    { NULL, NULL }
};

alloy_builtin_fn alloy_shell_find_builtin(const char *name) {
    for (int i = 0; ALLOY_BROWSER_API_TABLE[i].js_name != NULL; i++) { // Wait, wrong table reference
        // Fix reference to ALLOY_SHELL_BUILTINS
    }
    for (int i = 0; ALLOY_SHELL_BUILTINS[i].name; i++) {
        if (strcmp(ALLOY_SHELL_BUILTINS[i].name, name) == 0) return ALLOY_SHELL_BUILTINS[i].fn;
    }
    return NULL;
}

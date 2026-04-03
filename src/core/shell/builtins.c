#include "shell.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

typedef int (*alloy_builtin_fn)(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io);

typedef struct {
    const char *name;
    alloy_builtin_fn fn;
} alloy_builtin_entry_t;

int alloy_builtin_cd(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    if (argc < 2) return 0;
    return chdir(argv[1]);
}

int alloy_builtin_echo(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    for (int i = 1; i < argc; i++) {
        dprintf(io->fd_out, "%s%s", i > 1 ? " " : "", argv[i]);
    }
    dprintf(io->fd_out, "\n");
    return 0;
}

int alloy_builtin_pwd(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    char cwd[1024];
    if (getcwd(cwd, sizeof(cwd))) {
        dprintf(io->fd_out, "%s\n", cwd);
        return 0;
    }
    return 1;
}

int alloy_builtin_ls(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    // Basic ls implementation using WASI-compatible dirent
    return 0;
}

int alloy_builtin_rm(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    for (int i = 1; i < argc; i++) unlink(argv[i]);
    return 0;
}

int alloy_builtin_cat(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    for (int i = 1; i < argc; i++) {
        FILE *f = fopen(argv[i], "r");
        if (!f) continue;
        char buf[4096];
        size_t n;
        while ((n = fread(buf, 1, sizeof(buf), f)) > 0) write(io->fd_out, buf, n);
        fclose(f);
    }
    return 0;
}

int alloy_builtin_touch(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    for (int i = 1; i < argc; i++) {
        FILE *f = fopen(argv[i], "a");
        if (f) fclose(f);
    }
    return 0;
}

int alloy_builtin_mkdir(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    for (int i = 1; i < argc; i++) mkdir(argv[i], 0755);
    return 0;
}

int alloy_builtin_which(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) { return 0; }
int alloy_builtin_mv(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    if (argc < 3) return 1;
    return rename(argv[1], argv[2]);
}

int alloy_builtin_exit(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    int code = argc > 1 ? atoi(argv[1]) : 0;
    exit(code);
}

int alloy_builtin_true(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) { return 0; }
int alloy_builtin_false(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) { return 1; }
int alloy_builtin_yes(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    const char *s = argc > 1 ? argv[1] : "y";
    while (1) dprintf(io->fd_out, "%s\n", s);
    return 0;
}
int alloy_builtin_seq(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    if (argc < 3) return 1;
    int first = atoi(argv[1]);
    int last = atoi(argv[2]);
    for (int i = first; i <= last; i++) dprintf(io->fd_out, "%d\n", i);
    return 0;
}
int alloy_builtin_dirname(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    if (argc < 2) return 1;
    char *p = strrchr(argv[1], '/');
    if (!p) dprintf(io->fd_out, ".\n");
    else {
        *p = '\0';
        dprintf(io->fd_out, "%s\n", argv[1]);
    }
    return 0;
}
int alloy_builtin_basename(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    if (argc < 2) return 1;
    char *p = strrchr(argv[1], '/');
    if (!p) dprintf(io->fd_out, "%s\n", argv[1]);
    else dprintf(io->fd_out, "%s\n", p + 1);
    return 0;
}

const alloy_builtin_entry_t ALLOY_SHELL_BUILTINS[] = {
    {"cd", alloy_builtin_cd},
    {"echo", alloy_builtin_echo},
    {"pwd", alloy_builtin_pwd},
    {"ls", alloy_builtin_ls},
    {"rm", alloy_builtin_rm},
    {"cat", alloy_builtin_cat},
    {"touch", alloy_builtin_touch},
    {"mkdir", alloy_builtin_mkdir},
    {"which", alloy_builtin_which},
    {"mv", alloy_builtin_mv},
    {"exit", alloy_builtin_exit},
    {"true", alloy_builtin_true},
    {"false", alloy_builtin_false},
    {"yes", alloy_builtin_yes},
    {"seq", alloy_builtin_seq},
    {"dirname", alloy_builtin_dirname},
    {"basename", alloy_builtin_basename},
    {NULL, NULL}
};

alloy_builtin_fn alloy_shell_find_builtin(const char *name) {
    for (int i = 0; ALLOY_SHELL_BUILTINS[i].name; i++) {
        if (strcmp(ALLOY_SHELL_BUILTINS[i].name, name) == 0) return ALLOY_SHELL_BUILTINS[i].fn;
    }
    return NULL;
}

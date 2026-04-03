#include "builtins.h"
#include <string.h>
#include <stdio.h>
#include <unistd.h>
#include <stdlib.h>
#include <sys/stat.h>
#include <libgen.h>
#include <dirent.h>

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

static int builtin_ls(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    const char *path = argc > 1 ? argv[1] : ".";
    DIR *d = opendir(path);
    if (!d) return 1;
    struct dirent *dir;
    while ((dir = readdir(d)) != NULL) {
        if (dir->d_name[0] == '.') continue;
        dprintf(io->fd_out, "%s\n", dir->d_name);
    }
    closedir(d);
    return 0;
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
    char *p = strdup(argv[1]);
    dprintf(io->fd_out, "%s\n", dirname(p));
    free(p);
    return 0;
}

static int builtin_basename(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    if (argc < 2) return 1;
    char *p = strdup(argv[1]);
    dprintf(io->fd_out, "%s\n", basename(p));
    free(p);
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

static int builtin_rm(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    if (argc < 2) return 1;
    for (int i = 1; i < argc; i++) {
        if (unlink(argv[i]) != 0) return 1;
    }
    return 0;
}

static int builtin_cat(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    for (int i = 1; i < argc; i++) {
        FILE *f = fopen(argv[i], "r");
        if (!f) continue;
        char buf[1024];
        while (fgets(buf, sizeof(buf), f)) {
            dprintf(io->fd_out, "%s", buf);
        }
        fclose(f);
    }
    return 0;
}

static int builtin_touch(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    if (argc < 2) return 1;
    for (int i = 1; i < argc; i++) {
        FILE *f = fopen(argv[i], "a");
        if (f) fclose(f);
    }
    return 0;
}

static int builtin_mkdir(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    if (argc < 2) return 1;
    for (int i = 1; i < argc; i++) {
        if (mkdir(argv[i], 0755) != 0) return 1;
    }
    return 0;
}

static int builtin_mv(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    if (argc < 3) return 1;
    if (rename(argv[1], argv[2]) != 0) return 1;
    return 0;
}

static int builtin_yes(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    const char *s = argc > 1 ? argv[1] : "y";
    while (1) {
        dprintf(io->fd_out, "%s\n", s);
    }
    return 0;
}

static int builtin_which(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    if (argc < 2) return 1;
    // Mock path resolution
    dprintf(io->fd_out, "/usr/bin/%s\n", argv[1]);
    return 0;
}

const alloy_builtin_entry_t ALLOY_SHELL_BUILTINS[] = {
    { "echo", builtin_echo },
    { "cd", builtin_cd },
    { "pwd", builtin_pwd },
    { "ls", builtin_ls },
    { "rm", builtin_rm },
    { "cat", builtin_cat },
    { "touch", builtin_touch },
    { "mkdir", builtin_mkdir },
    { "mv", builtin_mv },
    { "which", builtin_which },
    { "exit", builtin_exit },
    { "true", builtin_true },
    { "false", builtin_false },
    { "yes", builtin_yes },
    { "seq", builtin_seq },
    { "dirname", builtin_dirname },
    { "basename", builtin_basename },
    { NULL, NULL }
};

alloy_builtin_fn alloy_shell_find_builtin(const char *name) {
    for (int i = 0; ALLOY_SHELL_BUILTINS[i].name; i++) {
        if (strcmp(ALLOY_SHELL_BUILTINS[i].name, name) == 0) return ALLOY_SHELL_BUILTINS[i].fn;
    }
    return NULL;
}

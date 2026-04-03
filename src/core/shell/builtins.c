#include "builtins.h"
#include <string.h>
#include <stdio.h>
#include <unistd.h>
#include <stdlib.h>
#include <sys/stat.h>
#include <dirent.h>
#include <fcntl.h>

// Forward declarations for all 17 builtins
static int builtin_cd(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io);
static int builtin_echo(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io);
static int builtin_pwd(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io);
static int builtin_ls(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io);
static int builtin_rm(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io);
static int builtin_cat(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io);
static int builtin_touch(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io);
static int builtin_mkdir(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io);
static int builtin_which(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io);
static int builtin_mv(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io);
static int builtin_exit(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io);
static int builtin_true(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io);
static int builtin_false(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io);
static int builtin_yes(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io);
static int builtin_seq(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io);
static int builtin_dirname(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io);
static int builtin_basename(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io);

const alloy_builtin_entry_t ALLOY_SHELL_BUILTINS[] = {
    {"cd", builtin_cd}, {"echo", builtin_echo}, {"pwd", builtin_pwd}, {"ls", builtin_ls},
    {"rm", builtin_rm}, {"cat", builtin_cat}, {"touch", builtin_touch}, {"mkdir", builtin_mkdir},
    {"which", builtin_which}, {"mv", builtin_mv}, {"exit", builtin_exit}, {"true", builtin_true},
    {"false", builtin_false}, {"yes", builtin_yes}, {"seq", builtin_seq}, {"dirname", builtin_dirname},
    {"basename", builtin_basename},
    {NULL, NULL}
};

alloy_builtin_fn alloy_shell_find_builtin(const char *name) {
    for (int i = 0; ALLOY_SHELL_BUILTINS[i].name; i++) {
        if (strcmp(ALLOY_SHELL_BUILTINS[i].name, name) == 0) return ALLOY_SHELL_BUILTINS[i].fn;
    }
    return NULL;
}

static int builtin_cd(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    if (argc < 2) return 0;
    if (chdir(argv[1]) < 0) {
        dprintf(io->fd_err, "cd: %s: No such directory\n", argv[1]);
        return 1;
    }
    return 0;
}

static int builtin_echo(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    for (int i = 1; i < argc; i++) {
        dprintf(io->fd_out, "%s%s", argv[i], (i == argc - 1) ? "" : " ");
    }
    dprintf(io->fd_out, "\n");
    return 0;
}

static int builtin_pwd(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    char cwd[4096];
    if (getcwd(cwd, sizeof(cwd))) {
        dprintf(io->fd_out, "%s\n", cwd);
        return 0;
    }
    return 1;
}

static int builtin_ls(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    const char *path = (argc > 1) ? argv[1] : ".";
    DIR *dir = opendir(path);
    if (!dir) return 1;
    struct dirent *ent;
    while ((ent = readdir(dir))) {
        if (ent->d_name[0] == '.') continue;
        dprintf(io->fd_out, "%s\n", ent->d_name);
    }
    closedir(dir);
    return 0;
}

static int builtin_rm(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    for (int i = 1; i < argc; i++) {
        if (unlink(argv[i]) < 0) {
            dprintf(io->fd_err, "rm: %s: Failed to remove\n", argv[i]);
            return 1;
        }
    }
    return 0;
}

static int builtin_cat(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    for (int i = 1; i < argc; i++) {
        int fd = open(argv[i], O_RDONLY);
        if (fd < 0) {
            dprintf(io->fd_err, "cat: %s: No such file\n", argv[i]);
            return 1;
        }
        char buf[4096];
        ssize_t n;
        while ((n = read(fd, buf, sizeof(buf))) > 0) {
            write(io->fd_out, buf, n);
        }
        close(fd);
    }
    return 0;
}

static int builtin_touch(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    for (int i = 1; i < argc; i++) {
        int fd = open(argv[i], O_WRONLY | O_CREAT, 0644);
        if (fd < 0) return 1;
        close(fd);
    }
    return 0;
}

static int builtin_mkdir(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    for (int i = 1; i < argc; i++) {
        if (mkdir(argv[i], 0755) < 0) return 1;
    }
    return 0;
}

static int builtin_which(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    if (argc < 2) return 1;
    if (alloy_shell_find_builtin(argv[1])) {
        dprintf(io->fd_out, "builtin: %s\n", argv[1]);
        return 0;
    }
    const char *path = getenv("PATH");
    if (!path) return 1;
    char *path_copy = strdup(path);
    char *dir = strtok(path_copy, ":");
    while (dir) {
        char full[4096];
        snprintf(full, sizeof(full), "%s/%s", dir, argv[1]);
        if (access(full, X_OK) == 0) {
            dprintf(io->fd_out, "%s\n", full);
            free(path_copy);
            return 0;
        }
        dir = strtok(NULL, ":");
    }
    free(path_copy);
    return 1;
}

static int builtin_mv(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    if (argc < 3) return 1;
    if (rename(argv[1], argv[2]) < 0) return 1;
    return 0;
}

static int builtin_exit(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    int code = (argc > 1) ? atoi(argv[1]) : 0;
    exit(code);
}

static int builtin_true(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) { return 0; }
static int builtin_false(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) { return 1; }

static int builtin_yes(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    const char *s = (argc > 1) ? argv[1] : "y";
    while (1) {
        if (dprintf(io->fd_out, "%s\n", s) < 0) break;
    }
    return 0;
}

static int builtin_seq(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    if (argc < 2) return 1;
    int first = 1, last;
    if (argc == 2) {
        last = atoi(argv[1]);
    } else {
        first = atoi(argv[1]);
        last = atoi(argv[2]);
    }
    for (int i = first; i <= last; i++) {
        dprintf(io->fd_out, "%d\n", i);
    }
    return 0;
}

static int builtin_dirname(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    if (argc < 2) return 1;
    char *p = strrchr(argv[1], '/');
    if (!p) { dprintf(io->fd_out, ".\n"); }
    else if (p == argv[1]) { dprintf(io->fd_out, "/\n"); }
    else { *p = '\0'; dprintf(io->fd_out, "%s\n", argv[1]); }
    return 0;
}

static int builtin_basename(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    if (argc < 2) return 1;
    char *p = strrchr(argv[1], '/');
    if (!p) { dprintf(io->fd_out, "%s\n", argv[1]); }
    else { dprintf(io->fd_out, "%s\n", p + 1); }
    return 0;
}

#include "builtins.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/stat.h>
#include <libgen.h>

/* Forward declaration of shell context structure */
struct alloy_shell_ctx {
    int throws;
    char **env;
    char *cwd;
    int exit_code;
};

static int builtin_cd(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    const char *path = argc > 1 ? argv[1] : getenv("HOME");
    if (!path) return 1;
    if (chdir(path) != 0) {
        dprintf(io->fd_err, "cd: %s: No such file or directory\n", path);
        return 1;
    }
    free(ctx->cwd);
    ctx->cwd = getcwd(NULL, 0);
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
    char *cwd = getcwd(NULL, 0);
    if (cwd) {
        dprintf(io->fd_out, "%s\n", cwd);
        free(cwd);
        return 0;
    }
    return 1;
}

#include <dirent.h>
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

static int builtin_rm(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    for (int i = 1; i < argc; i++) {
        if (unlink(argv[i]) != 0) {
            dprintf(io->fd_err, "rm: %s: No such file or directory\n", argv[i]);
            return 1;
        }
    }
    return 0;
}

static int builtin_cat(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    for (int i = 1; i < argc; i++) {
        FILE *f = fopen(argv[i], "r");
        if (!f) continue;
        char buf[1024];
        size_t n;
        while ((n = fread(buf, 1, sizeof(buf), f)) > 0) {
            write(io->fd_out, buf, n);
        }
        fclose(f);
    }
    return 0;
}

static int builtin_touch(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    for (int i = 1; i < argc; i++) {
        FILE *f = fopen(argv[i], "a");
        if (f) fclose(f);
    }
    return 0;
}

static int builtin_mkdir(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    for (int i = 1; i < argc; i++) {
        mkdir(argv[i], 0777);
    }
    return 0;
}

static int builtin_which(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    if (argc < 2) return 1;
    if (alloy_shell_find_builtin(argv[1])) {
        dprintf(io->fd_out, "builtin: %s\n", argv[1]);
        return 0;
    }
    // Search PATH (simplified stub for Wasm environment)
    char *path = getenv("PATH");
    if (path) {
        char *p = strdup(path);
        char *token = strtok(p, ":");
        while (token) {
            char full_path[1024];
            snprintf(full_path, sizeof(full_path), "%s/%s", token, argv[1]);
            if (access(full_path, X_OK) == 0) {
                dprintf(io->fd_out, "%s\n", full_path);
                free(p);
                return 0;
            }
            token = strtok(NULL, ":");
        }
        free(p);
    }
    return 1;
}

static int builtin_mv(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    if (argc < 3) return 1;
    return rename(argv[1], argv[2]);
}

static int builtin_exit(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    int code = argc > 1 ? atoi(argv[1]) : 0;
    exit(code);
    return code;
}

static int builtin_true(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) { return 0; }
static int builtin_false(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) { return 1; }

static int builtin_yes(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    const char *s = argc > 1 ? argv[1] : "y";
    while (1) {
        dprintf(io->fd_out, "%s\n", s);
    }
    return 0;
}

static int builtin_seq(alloy_shell_ctx_t *ctx, int argc, char **argv, alloy_stdio_t *io) {
    if (argc < 2) return 1;
    int start = 1, end = atoi(argv[1]);
    if (argc > 2) { start = atoi(argv[1]); end = atoi(argv[2]); }
    for (int i = start; i <= end; i++) {
        dprintf(io->fd_out, "%d\n", i);
    }
    return 0;
}

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

const alloy_builtin_entry_t ALLOY_SHELL_BUILTINS[] = {
    {"cd", builtin_cd}, {"echo", builtin_echo}, {"pwd", builtin_pwd}, {"ls", builtin_ls},
    {"rm", builtin_rm}, {"cat", builtin_cat}, {"touch", builtin_touch}, {"mkdir", builtin_mkdir},
    {"which", builtin_which}, {"mv", builtin_mv}, {"exit", builtin_exit}, {"true", builtin_true},
    {"false", builtin_false}, {"yes", builtin_yes}, {"seq", builtin_seq}, {"dirname", builtin_dirname},
    {"basename", builtin_basename}, {NULL, NULL}
};

alloy_builtin_fn alloy_shell_find_builtin(const char *name) {
    for (int i = 0; ALLOY_SHELL_BUILTINS[i].name; i++) {
        if (strcmp(ALLOY_SHELL_BUILTINS[i].name, name) == 0) return ALLOY_SHELL_BUILTINS[i].fn;
    }
    return NULL;
}

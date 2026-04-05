#include "builtins.h"
#include <string.h>
#include <unistd.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/stat.h>

static int builtin_echo(int argc, char **argv, alloy_stdio_t *io) {
    int out = io->fd_out == -1 ? STDOUT_FILENO : io->fd_out;
    for (int i = 1; i < argc; i++) {
        write(out, argv[i], strlen(argv[i]));
        if (i < argc - 1) write(out, " ", 1);
    }
    write(out, "\n", 1);
    return 0;
}

static int builtin_cat(int argc, char **argv, alloy_stdio_t *io) {
    int out = io->fd_out == -1 ? STDOUT_FILENO : io->fd_out;
    for (int i = 1; i < argc; i++) {
        FILE *f = fopen(argv[i], "r");
        if (!f) continue;
        char buf[4096];
        size_t n;
        while ((n = fread(buf, 1, sizeof(buf), f)) > 0) write(out, buf, n);
        fclose(f);
    }
    return 0;
}

static int builtin_ls(int argc, char **argv, alloy_stdio_t *io) {
    int out = io->fd_out == -1 ? STDOUT_FILENO : io->fd_out;
    write(out, ".\n..\n", 5); /* Minimal simulation */
    return 0;
}

static int builtin_dirname(int argc, char **argv, alloy_stdio_t *io) {
    if (argc < 2) return 1;
    int out = io->fd_out == -1 ? STDOUT_FILENO : io->fd_out;
    char *p = strrchr(argv[1], '/');
    if (p) {
        write(out, argv[1], p - argv[1]);
        write(out, "\n", 1);
    } else {
        write(out, ".\n", 2);
    }
    return 0;
}

static int builtin_basename(int argc, char **argv, alloy_stdio_t *io) {
    if (argc < 2) return 1;
    int out = io->fd_out == -1 ? STDOUT_FILENO : io->fd_out;
    char *p = strrchr(argv[1], '/');
    if (p) write(out, p + 1, strlen(p + 1));
    else write(out, argv[1], strlen(argv[1]));
    write(out, "\n", 1);
    return 0;
}

static int builtin_yes(int argc, char **argv, alloy_stdio_t *io) {
    int out = io->fd_out == -1 ? STDOUT_FILENO : io->fd_out;
    const char *s = (argc > 1) ? argv[1] : "y";
    while (1) {
        write(out, s, strlen(s));
        write(out, "\n", 1);
    }
    return 0;
}

static int builtin_rm(int argc, char **argv, alloy_stdio_t *io) {
    for (int i = 1; i < argc; i++) unlink(argv[i]);
    return 0;
}

static int builtin_mkdir(int argc, char **argv, alloy_stdio_t *io) {
    if (argc < 2) return 1;
    return mkdir(argv[1], 0755);
}

static int builtin_touch(int argc, char **argv, alloy_stdio_t *io) {
    for (int i = 1; i < argc; i++) {
        FILE *f = fopen(argv[i], "a");
        if (f) fclose(f);
    }
    return 0;
}

static int builtin_mv(int argc, char **argv, alloy_stdio_t *io) {
    if (argc < 3) return 1;
    return rename(argv[1], argv[2]);
}

static int builtin_exit(int argc, char **argv, alloy_stdio_t *io) {
    int code = (argc > 1) ? atoi(argv[1]) : 0;
    exit(code);
}

static int builtin_seq(int argc, char **argv, alloy_stdio_t *io) {
    if (argc < 3) return 1;
    int start = atoi(argv[1]);
    int end = atoi(argv[2]);
    int out = io->fd_out == -1 ? STDOUT_FILENO : io->fd_out;
    for (int i = start; i <= end; i++) {
        char buf[32];
        int len = snprintf(buf, sizeof(buf), "%d\n", i);
        write(out, buf, len);
    }
    return 0;
}

static int builtin_pwd(int argc, char **argv, alloy_stdio_t *io) {
    char cwd[1024];
    if (getcwd(cwd, sizeof(cwd))) {
        write(io->fd_out == -1 ? STDOUT_FILENO : io->fd_out, cwd, strlen(cwd));
        write(io->fd_out == -1 ? STDOUT_FILENO : io->fd_out, "\n", 1);
        return 0;
    }
    return 1;
}

static int builtin_cd(int argc, char **argv, alloy_stdio_t *io) {
    if (argc < 2) return 1;
    return chdir(argv[1]);
}

static int builtin_true(int argc, char **argv, alloy_stdio_t *io) { return 0; }
static int builtin_false(int argc, char **argv, alloy_stdio_t *io) { return 1; }

static const alloy_builtin_entry_t ALLOY_SHELL_BUILTINS[] = {
    { "echo",     builtin_echo },
    { "pwd",      builtin_pwd },
    { "cd",       builtin_cd },
    { "cat",      builtin_cat },
    { "rm",       builtin_rm },
    { "ls",       builtin_ls },
    { "mkdir",    builtin_mkdir },
    { "touch",    builtin_touch },
    { "mv",       builtin_mv },
    { "dirname",  builtin_dirname },
    { "basename", builtin_basename },
    { "yes",      builtin_yes },
    { "exit",     builtin_exit },
    { "seq",      builtin_seq },
    { "true",     builtin_true },
    { "false",    builtin_false },
    { NULL,       NULL }
};

alloy_builtin_fn alloy_shell_find_builtin(const char *name) {
    for (int i = 0; ALLOY_SHELL_BUILTINS[i].name; i++) {
        if (strcmp(ALLOY_SHELL_BUILTINS[i].name, name) == 0) return ALLOY_SHELL_BUILTINS[i].fn;
    }
    return NULL;
}

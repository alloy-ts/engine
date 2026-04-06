#include "builtins.h"
#include <string.h>
#include <unistd.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/stat.h>
#include <fcntl.h>

static int builtin_echo(int argc, char **argv, alloy_stdio_t *io) {
    int out = (io && io->fd_out != -1) ? io->fd_out : STDOUT_FILENO;
    for (int i = 1; i < argc; i++) {
        write(out, argv[i], strlen(argv[i]));
        if (i < argc - 1) write(out, " ", 1);
    }
    write(out, "\n", 1);
    return 0;
}

static int builtin_cat(int argc, char **argv, alloy_stdio_t *io) {
    int out = (io && io->fd_out != -1) ? io->fd_out : STDOUT_FILENO;
    int in = (io && io->fd_in != -1) ? io->fd_in : STDIN_FILENO;

    if (argc == 1) {
        char buf[4096];
        ssize_t n;
        while ((n = read(in, buf, sizeof(buf))) > 0) write(out, buf, n);
        return 0;
    }

    for (int i = 1; i < argc; i++) {
        int fd = open(argv[i], O_RDONLY);
        if (fd < 0) continue;
        char buf[4096];
        ssize_t n;
        while ((n = read(fd, buf, sizeof(buf))) > 0) write(out, buf, n);
        close(fd);
    }
    return 0;
}

#include <dirent.h>

static int builtin_ls(int argc, char **argv, alloy_stdio_t *io) {
    int out = (io && io->fd_out != -1) ? io->fd_out : STDOUT_FILENO;
    const char *path = (argc > 1) ? argv[1] : ".";
    DIR *d = opendir(path);
    if (!d) return 1;
    struct dirent *dir;
    while ((dir = readdir(d)) != NULL) {
        write(out, dir->d_name, strlen(dir->d_name));
        write(out, "\n", 1);
    }
    closedir(d);
    return 0;
}

static int builtin_dirname(int argc, char **argv, alloy_stdio_t *io) {
    if (argc < 2) return 1;
    int out = (io && io->fd_out != -1) ? io->fd_out : STDOUT_FILENO;
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
    int out = (io && io->fd_out != -1) ? io->fd_out : STDOUT_FILENO;
    char *p = strrchr(argv[1], '/');
    if (p) write(out, p + 1, strlen(p + 1));
    else write(out, argv[1], strlen(argv[1]));
    write(out, "\n", 1);
    return 0;
}

static int builtin_yes(int argc, char **argv, alloy_stdio_t *io) {
    int out = (io && io->fd_out != -1) ? io->fd_out : STDOUT_FILENO;
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
    int out = (io && io->fd_out != -1) ? io->fd_out : STDOUT_FILENO;
    for (int i = start; i <= end; i++) {
        char buf[32];
        int len = snprintf(buf, sizeof(buf), "%d\n", i);
        write(out, buf, len);
    }
    return 0;
}

static int builtin_pwd(int argc, char **argv, alloy_stdio_t *io) {
    char cwd[1024];
    int out = (io && io->fd_out != -1) ? io->fd_out : STDOUT_FILENO;
    if (getcwd(cwd, sizeof(cwd))) {
        write(out, cwd, strlen(cwd));
        write(out, "\n", 1);
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

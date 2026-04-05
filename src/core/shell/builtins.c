#include "builtins.h"
#include <stdio.h>
#include <string.h>
#include <unistd.h>
#include <stdlib.h>

typedef struct {
    const char *name;
    BuiltinFn fn;
} BuiltinEntry;

static BuiltinEntry builtins[] = {
    { "cd", builtin_cd },
    { "echo", builtin_echo },
    { "pwd", builtin_pwd },
    { "ls", builtin_ls },
    { "rm", builtin_rm },
    { "cat", builtin_cat },
    { "touch", builtin_touch },
    { "mkdir", builtin_mkdir },
    { "exit", builtin_exit },
    { "true", builtin_true },
    { "false", builtin_false },
    { NULL, NULL }
};

BuiltinFn builtin_find(const char *name) {
    for (int i = 0; builtins[i].name; i++) {
        if (strcmp(builtins[i].name, name) == 0) return builtins[i].fn;
    }
    return NULL;
}

int builtin_echo(int argc, char **argv) {
    for (int i = 1; i < argc; i++) {
        printf("%s%c", argv[i], i == argc - 1 ? '\n' : ' ');
    }
    if (argc == 1) printf("\n");
    return 0;
}

int builtin_cd(int argc, char **argv) {
    if (argc < 2) return 0;
    return chdir(argv[1]);
}

int builtin_pwd(int argc, char **argv) {
    char buf[1024];
    if (getcwd(buf, sizeof(buf))) {
        printf("%s\n", buf);
        return 0;
    }
    return 1;
}

int builtin_true(int argc, char **argv) { return 0; }
int builtin_false(int argc, char **argv) { return 1; }
int builtin_exit(int argc, char **argv) {
    int code = 0;
    if (argc > 1) code = atoi(argv[1]);
    exit(code);
    return code;
}

#include <dirent.h>
#include <sys/stat.h>

int builtin_ls(int argc, char **argv) {
    const char *path = argc > 1 ? argv[1] : ".";
    DIR *d = opendir(path);
    if (!d) { perror("ls"); return 1; }
    struct dirent *dir;
    while ((dir = readdir(d)) != NULL) {
        if (dir->d_name[0] != '.') printf("%s  ", dir->d_name);
    }
    printf("\n");
    closedir(d);
    return 0;
}

int builtin_rm(int argc, char **argv) {
    if (argc < 2) return 0;
    for (int i = 1; i < argc; i++) {
        if (unlink(argv[i]) != 0) perror(argv[i]);
    }
    return 0;
}

int builtin_cat(int argc, char **argv) {
    if (argc < 2) return 0;
    for (int i = 1; i < argc; i++) {
        FILE *f = fopen(argv[i], "r");
        if (!f) { perror(argv[i]); continue; }
        char buf[4096];
        size_t n;
        while ((n = fread(buf, 1, sizeof(buf), f)) > 0) fwrite(buf, 1, n, stdout);
        fclose(f);
    }
    return 0;
}

int builtin_touch(int argc, char **argv) {
    if (argc < 2) return 0;
    for (int i = 1; i < argc; i++) {
        FILE *f = fopen(argv[i], "a");
        if (f) fclose(f);
        else perror(argv[i]);
    }
    return 0;
}

int builtin_mkdir(int argc, char **argv) {
    if (argc < 2) return 0;
    for (int i = 1; i < argc; i++) {
        if (mkdir(argv[i], 0755) != 0) perror(argv[i]);
    }
    return 0;
}

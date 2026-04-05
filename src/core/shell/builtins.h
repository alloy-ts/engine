#ifndef ALLOY_SHELL_BUILTINS_H
#define ALLOY_SHELL_BUILTINS_H

typedef int (*BuiltinFn)(int argc, char **argv);

BuiltinFn builtin_find(const char *name);

int builtin_cd(int argc, char **argv);
int builtin_echo(int argc, char **argv);
int builtin_pwd(int argc, char **argv);
int builtin_ls(int argc, char **argv);
int builtin_rm(int argc, char **argv);
int builtin_cat(int argc, char **argv);
int builtin_touch(int argc, char **argv);
int builtin_mkdir(int argc, char **argv);
int builtin_exit(int argc, char **argv);
int builtin_true(int argc, char **argv);
int builtin_false(int argc, char **argv);

#endif

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    char *command;
} whitelist_entry_t;

static whitelist_entry_t whitelist[] = {
    {"git"}, {"ls"}, {"cat"}, {"echo"}, {"pwd"}, {"rm"}, {"touch"}, {"mkdir"}, {"mv"},
    {NULL}
};

int alloy_shell_wit_enforce_whitelist(const char *command) {
    for (int i = 0; whitelist[i].command; i++) {
        if (strcmp(whitelist[i].command, command) == 0) return 1;
    }
    return 0;
}

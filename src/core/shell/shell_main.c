#include <stdio.h>
#include <stdlib.h>

void alloy_shell_exec(const char *script);

int main(int argc, char **argv) {
    /* Requirement 1.9: shell.wasm entry point */
    if (argc > 1) {
        alloy_shell_exec(argv[1]);
    } else {
        printf("Alloy Shell Interpreter\n");
    }
    return 0;
}

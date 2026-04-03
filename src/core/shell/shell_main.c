#include "shell.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

int main() {
    char buf[4096];
    if (fgets(buf, sizeof(buf), stdin)) {
        alloy_shell_ctx_t ctx = {0};
        alloy_shell_result_t res = {0};

        // Placeholder JSON parsing
        alloy_shell_exec(&ctx, buf, &res);

        // Write JSON to stdout
        printf("{\"exitCode\": %d, \"stdout\": \"\", \"stderr\": \"\"}\n", res.exit_code);
        alloy_shell_result_free(&res);
    }
    return 0;
}

#include "shell.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// Placeholder for JSON library (like cJSON)
int main() {
    char input[4096];
    fgets(input, sizeof(input), stdin);

    alloy_shell_ctx_t ctx = {0};
    alloy_shell_result_t result = {0};

    // Parse script from input JSON
    const char *script = "echo hello";
    alloy_shell_exec(&ctx, script, &result);

    printf("{\"exitCode\": %d, \"stdout\": \"\", \"stderr\": \"\"}\n", result.exit_code);
    return 0;
}

#include "shell.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// Standard C entry point for the shell, compiled with host toolchain
int main(int argc, char **argv) {
    char buf[4096];
    if (!fgets(buf, sizeof(buf), stdin)) return 1;

    // JSON parsing simplified
    char *script_ptr = strstr(buf, "\"script\":");
    if (!script_ptr) return 1;
    script_ptr = strchr(script_ptr, '"') + 1;
    char *script_end = strchr(script_ptr + 8, '"');
    if (!script_end) return 1;
    *script_end = '\0';

    alloy_shell_ctx_t ctx = { {1, NULL, NULL}, NULL, 0 };
    alloy_shell_result_t result;
    alloy_shell_exec(&ctx, script_ptr, &result);

    // In native path, we write JSON to stdout just like shell.wasm
    char *stdout_base64 = "base64(stdout)"; // placeholder
    char *stderr_base64 = "base64(stderr)"; // placeholder

    printf("{\"exitCode\": %d, \"stdout\": \"%s\", \"stderr\": \"%s\"}\n",
           result.exit_code, stdout_base64, stderr_base64);

    return 0;
}

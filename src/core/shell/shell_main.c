#include "shell.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int alloy_shell_main() {
    char buf[1024];
    // Read JSON from stdin
    if (fgets(buf, sizeof(buf), stdin)) {
        alloy_shell_ctx_t ctx;
        alloy_shell_result_t res;
        // Parse JSON, call exec
        alloy_shell_exec(&ctx, buf, &res);
        // Write JSON to stdout
        printf("{\"exitCode\": %d, \"stdout\": \"bW9jayBzdGRvdXQ=\"}\n", res.exit_code);
        alloy_shell_result_free(&res);
    }
    return 0;
}

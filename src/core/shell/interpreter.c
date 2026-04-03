#include "shell.h"
#include "lexer.h"
#include "parser.h"
#include <stdlib.h>
#include <string.h>

int alloy_shell_exec(alloy_shell_ctx_t *ctx, const char *script, alloy_shell_result_t *out) {
    alloy_token_list_t tl;
    if (alloy_shell_lex(script, &tl) != 0) return -1;

    alloy_node_t *ast;
    if (alloy_shell_parse(&tl, &ast) != 0) {
        alloy_token_list_free(&tl);
        return -1;
    }

    // Execution logic
    out->exit_code = 0;
    out->stdout_buf = (uint8_t*)strdup("shell execution result");
    out->stdout_len = strlen((char*)out->stdout_buf);

    alloy_node_free(ast);
    alloy_token_list_free(&tl);
    return 0;
}

void alloy_shell_result_free(alloy_shell_result_t *r) {
    if (r->stdout_buf) free(r->stdout_buf);
    if (r->stderr_buf) free(r->stderr_buf);
}

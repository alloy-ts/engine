#include "lexer.h"
#include "parser.h"
#include "interpreter.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(int argc, char **argv) {
    char buffer[4096];
    if (fgets(buffer, sizeof(buffer), stdin) == NULL) return 1;

    // In a real implementation, this would parse JSON { script, env, cwd }
    // For foundation, we'll assume the script is passed directly as a string in stdin

    Lexer *l = lexer_create(buffer);
    Node *ast = parser_parse(l);

    int exit_code = interpreter_exec(ast);

    // Output JSON: { "exitCode": X, "stdout": "...", "stderr": "..." }
    printf("{\"exitCode\": %d}\n", exit_code);

    node_free(ast);
    lexer_free(l);

    return 0;
}

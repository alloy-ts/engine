#include "lexer.h"
#include "parser.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

void alloy_shell_exec(node_t *n) {
    if (n->type == NODE_SCRIPT) {
        for (int i = 0; i < n->num_children; i++) {
            alloy_shell_exec(n->children[i]);
        }
    } else if (n->type == NODE_COMMAND) {
        if (n->argc > 0) {
            printf("Executing: %s\n", n->argv[0]);
            // In Wasm we would use WASI fds/threads/pipes
        }
    }
}

int main(int argc, char **argv) {
    // Read JSON from stdin for real use
    lexer_t l = { "echo hello; ls -la", 0 };
    node_t *ast = parse_script(&l);
    alloy_shell_exec(ast);
    return 0;
}

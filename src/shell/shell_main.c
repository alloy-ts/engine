#include "lexer.h"
#include "parser.h"
#include "builtins.h"
#include <stdio.h>
#include <stdlib.h>

extern int alloy_shell_exec_node(alloy_node_t *node, alloy_stdio_t *io);

int main(int argc, char **argv) {
    if (argc < 2) {
        fprintf(stderr, "Usage: alloy_shell <script>\n");
        return 1;
    }

    alloy_token_list_t tl;
    alloy_shell_lex(argv[1], &tl);

    alloy_node_t *root;
    alloy_shell_parse(&tl, &root);

    alloy_stdio_t io = {0, 1, 2};
    int res = alloy_shell_exec_node(root, &io);

    alloy_node_free(root);
    alloy_token_list_free(&tl);

    return res;
}

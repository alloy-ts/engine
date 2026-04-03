#include "parser.h"
#include "lexer.h"
#include <assert.h>
#include <stdio.h>

void test_parser() {
    alloy_token_list_t tl;
    alloy_shell_lex("echo hello | cat > file.txt", &tl);

    alloy_node_t *root;
    alloy_shell_parse(&tl, &root);

    assert(root->type == NODE_SCRIPT);
    assert(root->script.count == 1);

    alloy_node_t *pipeline = root->script.pipelines[0];
    assert(pipeline->type == NODE_PIPELINE);
    assert(pipeline->pipeline.stage_count == 2);

    alloy_node_t *cmd1 = pipeline->pipeline.stages[0];
    assert(cmd1->type == NODE_COMMAND);
    assert(cmd1->command.argc == 2);

    alloy_node_t *cmd2 = pipeline->pipeline.stages[1];
    assert(cmd2->type == NODE_COMMAND);
    assert(cmd2->command.argc == 1);
    assert(cmd2->command.redir_count == 1);

    alloy_node_free(root);
    alloy_token_list_free(&tl);
    printf("test_parser passed\n");
}

int main() {
    test_parser();
    return 0;
}

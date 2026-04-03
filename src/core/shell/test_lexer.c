#include "lexer.h"
#include <assert.h>
#include <string.h>
#include <stdio.h>

void test_lexer() {
    alloy_token_list_t tl;
    alloy_shell_lex("echo hello | cat > file.txt", &tl);

    assert(tl.count == 7);
    assert(tl.tokens[0].type == TOK_WORD);
    assert(strcmp(tl.tokens[0].value, "echo") == 0);
    assert(tl.tokens[1].type == TOK_WORD);
    assert(strcmp(tl.tokens[1].value, "hello") == 0);
    assert(tl.tokens[2].type == TOK_PIPE);
    assert(tl.tokens[3].type == TOK_WORD);
    assert(strcmp(tl.tokens[3].value, "cat") == 0);
    assert(tl.tokens[4].type == TOK_REDIR_OUT);
    assert(tl.tokens[5].type == TOK_WORD);
    assert(strcmp(tl.tokens[5].value, "file.txt") == 0);
    assert(tl.tokens[6].type == TOK_EOF);

    alloy_token_list_free(&tl);
    printf("test_lexer passed\n");
}

int main() {
    test_lexer();
    return 0;
}

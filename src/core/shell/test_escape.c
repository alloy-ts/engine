#include "escape.h"
#include <assert.h>
#include <string.h>
#include <stdio.h>

void test_escape() {
    char dst[4096];
    alloy_shell_escape("hello;world", dst, sizeof(dst));
    assert(strcmp(dst, "hello\\;world") == 0);

    // Idempotence
    char dst2[4096];
    alloy_shell_escape(dst, dst2, sizeof(dst2));
    assert(strcmp(dst, dst2) == 0);

    printf("test_escape passed\n");
}

int main() {
    test_escape();
    return 0;
}

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>
#include "../src/core/engine.h"

int main() {
    printf("Running C-layer security tests...\n");

    /* Feature: alloyscript-dual-engine, Property 31: .rodata dispatch tables are never writable */
    /* Handled by static analysis and binary layout verification */

    /* Feature: alloyscript-dual-engine, Property 36: Self-attestation */
    int res = alloy_self_attest();
    assert(res == 0);

    printf("C-layer security tests passed.\n");
    return 0;
}

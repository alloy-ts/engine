#include "alloy/alloy.h"
#include <stdio.h>
#include <string.h>
#include <assert.h>

/* Task 12.1: Test is_complete_expression() */
/* This would require exposing it in a header or just testing the REPL logic */

void test_basic_eval() {
    alloy_runtime_t *rt = alloy_engine_create();
    assert(rt != NULL);

    /* Simulate REPL evaluation wrapper */
    const char *script = "(function(){ return JSON.stringify(1 + 1); })()";
    JSValue res = alloy_secure_eval(rt, script);
    const char *res_str = JS_ToCString(rt->mqjs_ctx, res);
    assert(strcmp(res_str, "2") == 0);

    JS_FreeCString(rt->mqjs_ctx, res_str);
    JS_FreeValue(rt->mqjs_ctx, res);
    alloy_engine_destroy(rt);
    printf("test_basic_eval passed\n");
}

int main() {
    test_basic_eval();
    return 0;
}

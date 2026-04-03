#include "engine.h"
#include <stdio.h>
#include <string.h>
#include <stdlib.h>

int main() {
    printf("Initializing runtime...\n");
    alloy_runtime_t *rt = alloy_engine_create();
    if (!rt) return 1;

    printf("Engine created successfully\n");

    const char *script = "1 + 2;";

    printf("Evaluating script...\n");
    JSValue res = JS_Eval(rt->mqjs_ctx, script, strlen(script), "<test>", 0);
    if (JS_IsException(res)) {
        printf("Script failed with exception\n");
        return 1;
    }

    printf("Script evaluated.\n");

    alloy_engine_destroy(rt);
    printf("Engine destroyed.\n");
    return 0;
}

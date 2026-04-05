#include "engine.h"
#include <stdio.h>
#include <string.h>
#include <stdlib.h>

int main() {
    alloy_runtime_t *rt = alloy_engine_create();
    if (!rt) return 1;

    JSValue res = JS_Eval(rt->mqjs_ctx, "typeof JSON", 11, "<test>", JS_EVAL_RETVAL);
    JSCStringBuf buf;
    const char *cstr = JS_ToCString(rt->mqjs_ctx, res, &buf);
    printf("Type of JSON: %s\n", cstr);

    alloy_engine_destroy(rt);
    return 0;
}

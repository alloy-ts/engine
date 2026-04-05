#include "engine.h"
#include <stdio.h>
#include <string.h>
#include <stdlib.h>

char* test_cb(const char *id, const char *req, void *arg) {
    printf("C callback called with: %s\n", req);
    return strdup("\"hello from C\"");
}

int main() {
    printf("Initializing runtime...\n");
    alloy_runtime_t *rt = alloy_engine_create();
    if (!rt) return 1;

    printf("Engine created successfully\n");

    // Test JS -> C binding
    rt->core_engine->bind_fn(rt->core_engine, "test_func", test_cb, NULL);

    // Test C -> JS call via eval
    printf("Testing C -> JS call...\n");
    rt->core_engine->eval_fn(rt->core_engine, "var js_func = function(x) { return 'JS received: ' + x; };");
    char *res = rt->core_engine->call_fn(rt->core_engine, "js_func", "[\"world\"]");
    printf("Result from JS: %s\n", res);
    if (res) free(res);

    // Test JS calling bound C function
    printf("Testing JS -> C call...\n");
    rt->core_engine->eval_fn(rt->core_engine, "c_res = test_func('from JS'); last_c_res = c_res;");
    char *c_res_in_js = rt->core_engine->call_fn(rt->core_engine, "eval", "['last_c_res']");
    printf("C result in JS: %s\n", c_res_in_js);
    if (c_res_in_js) free(c_res_in_js);

    alloy_engine_destroy(rt);
    printf("Engine destroyed.\n");
    return 0;
}

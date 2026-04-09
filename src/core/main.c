#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "alloy/alloy.h"
int main(int argc, char *argv[]) {
    if (argc >= 2 && strcmp(argv[1], "repl") == 0) { extern int alloy_repl_run(int, char**); return alloy_repl_run(argc-1, argv+1); }
    if (argc < 2) return 1;
    AlloyRuntime *rt = alloy_engine_create(); if (!rt) return 1;
    FILE *f = fopen(argv[1], "r"); if (f) {
        fseek(f, 0, SEEK_END); long size = ftell(f); fseek(f, 0, SEEK_SET);
        char *script = malloc(size + 1); fread(script, 1, size, f); script[size] = 0; fclose(f);
        JSValue val = JS_Eval(get_rt_ctx(rt), script, size, argv[1], 0);
        if (JS_IsException(val)) { JS_PrintValueF(get_rt_ctx(rt), JS_GetException(get_rt_ctx(rt)), 1); printf("\n"); }
        free(script);
    }
    alloy_engine_destroy(rt); return 0;
}

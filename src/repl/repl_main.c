#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#include "alloy/alloy.h"
#include "mquickjs.h"

typedef struct {
    AlloyRuntime *rt;
    char multiline_buf[65536];
    int multiline_len;
} ReplState;

static bool is_complete_expression(const char *src, int len) {
    int depth = 0;
    for (int i = 0; i < len; i++) {
        if (src[i] == '{' || src[i] == '(' || src[i] == '[') depth++;
        if (src[i] == '}' || src[i] == ')' || src[i] == ']') depth--;
    }
    return depth <= 0;
}

int alloy_repl_run(int argc, char **argv) {
    printf("AlloyScript REPL - Phase 1\n");
    printf("(web APIs available on demand)\n");

    AlloyRuntime *rt = alloy_engine_create();
    if (!rt) return 1;

    char line[4096];
    ReplState state = { .rt = rt, .multiline_len = 0 };

    while (printf(state.multiline_len > 0 ? "... " : "> ") && fflush(stdout) && fgets(line, sizeof(line), stdin)) {
        if (line[0] == '.') {
            if (strncmp(line, ".exit", 5) == 0) break;
            if (strncmp(line, ".help", 5) == 0) {
                printf(".exit - Exit REPL\n.help - Show help\n.clear - Clear buffer\n");
                continue;
            }
            if (strncmp(line, ".clear", 6) == 0) {
                state.multiline_len = 0;
                continue;
            }
        }

        size_t len = strlen(line);
        if (state.multiline_len + len < sizeof(state.multiline_buf)) {
            memcpy(state.multiline_buf + state.multiline_len, line, len);
            state.multiline_len += len;
            state.multiline_buf[state.multiline_len] = 0;
        }

        if (is_complete_expression(state.multiline_buf, state.multiline_len)) {
            JSContext *ctx = get_rt_ctx(rt);
            JSValue val = JS_Eval(ctx, state.multiline_buf, state.multiline_len, "<repl>", 0);
            if (JS_IsException(val)) {
                JSValue err = JS_GetException(ctx);
                printf("! ");
                JS_PrintValueF(ctx, err, 1);
                printf("\n");
            } else if (!JS_IsUndefined(val)) {
                printf("< ");
                JS_PrintValueF(ctx, val, 1);
                printf("\n");
            }
            state.multiline_len = 0;
        }
    }

    alloy_engine_destroy(rt);
    return 0;
}

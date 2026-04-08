#include "alloy/alloy.h"
#include "plugin.h"
#include "packages/mquickjs/readline.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#include <unistd.h>
#include <sys/types.h>

/* ReplState definition - Requirement 1.4 */
typedef struct {
    alloy_runtime_t *rt;
    char             multiline_buf[65536];
    int              multiline_len;
    bool             no_web;
    FILE            *history_file;
    char             history_path[4096];
} ReplState;

/* Forward declarations */
static bool is_complete_expression(const char *src, int len);
static void print_result(JSContext *ctx, const char *json_result);
static bool dispatch_special_command(ReplState *s, const char *cmd);

int alloy_repl_run(int argc, char **argv) {
    ReplState state = {0};

    /* Parse --no-web - Requirement 8.3 */
    for (int i = 0; i < argc; i++) {
        if (strcmp(argv[i], "--no-web") == 0) {
            state.no_web = true;
        }
    }

    /* Create engine - Requirement 1.1 */
    state.rt = alloy_engine_create();
    if (!state.rt) {
        fprintf(stderr, "[alloy] fatal: failed to create runtime\n");
        return 1;
    }

    /* Print banner - Requirement 1.5 */
    printf("AlloyScript REPL v1.0.0 (web APIs available on demand)\n");

    /* Initialize history path */
    const char *home = getenv("HOME");
    if (home) {
        snprintf(state.history_path, sizeof(state.history_path), "%s/.alloy_repl_history", home);
        readline_load_history(state.history_path);
    }

    char *line;
    while ((line = readline(state.multiline_len > 0 ? "... " : "> ")) != NULL) {
        if (line[0] == '.') {
            if (dispatch_special_command(&state, line)) {
                readline_add_history(line);
                free(line);
                continue;
            }
        }

        /* Multiline handling - Requirement 4.3 */
        if (state.multiline_len + strlen(line) + 2 > sizeof(state.multiline_buf)) {
            fprintf(stderr, "error: multiline buffer overflow\n");
            state.multiline_len = 0;
            free(line);
            continue;
        }

        if (state.multiline_len > 0) {
            strcat(state.multiline_buf, line);
        } else {
            strcpy(state.multiline_buf, line);
        }
        state.multiline_len += strlen(line);
        state.multiline_buf[state.multiline_len++] = '\n';
        state.multiline_buf[state.multiline_len] = '\0';

        if (is_complete_expression(state.multiline_buf, state.multiline_len)) {
            /* Evaluation wrapper - Requirement 5.1 */
            char eval_script[65536 + 512];
            snprintf(eval_script, sizeof(eval_script),
                "(function(){ try { var r=(%s); if(typeof r==='undefined') return '__alloy_undefined__'; return JSON.stringify(r); } catch(e){ return '! ' + e.toString(); } })()",
                state.multiline_buf);

            JSValue res = alloy_secure_eval(state.rt, eval_script);
            if (!JS_IsException(res)) {
                const char *res_json = JS_ToCString(state.rt->mqjs_ctx, res);
                print_result(state.rt->mqjs_ctx, res_json);
                JS_FreeCString(state.rt->mqjs_ctx, res_json);
            } else {
                JSValue err = JS_GetException(state.rt->mqjs_ctx);
                const char *err_str = JS_ToCString(state.rt->mqjs_ctx, err);
                fprintf(stderr, "! %s\n", err_str);
                JS_FreeCString(state.rt->mqjs_ctx, err_str);
                JS_FreeValue(state.rt->mqjs_ctx, err);
            }
            JS_FreeValue(state.rt->mqjs_ctx, res);

            readline_add_history(state.multiline_buf);
            if (state.history_path[0]) readline_save_history(state.history_path);

            state.multiline_len = 0;
            state.multiline_buf[0] = '\0';
        }

        free(line);
    }

    alloy_engine_destroy(state.rt);
    return 0;
}

static bool is_complete_expression(const char *src, int len) {
    int depth = 0;
    bool in_template = false;
    for (int i = 0; i < len; i++) {
        if (src[i] == '`') in_template = !in_template;
        if (in_template) continue;
        if (src[i] == '{' || src[i] == '(' || src[i] == '[') depth++;
        if (src[i] == '}' || src[i] == ')' || src[i] == ']') depth--;
    }
    return depth <= 0 && !in_template;
}

static void print_result(JSContext *ctx, const char *json_result) {
    if (strcmp(json_result, "__alloy_undefined__") == 0) return;
    if (strncmp(json_result, "! ", 2) == 0) {
        fprintf(stderr, "%s\n", json_result);
        return;
    }

    /* Requirement 5.4: Pretty-print JSON if object/array */
    if (json_result[0] == '{' || json_result[0] == '[') {
        /* Simple indentation for stub, in production we'd parse and format properly */
        printf("< %s\n", json_result);
    } else if (json_result[0] == '"') {
        /* Requirement 5.5: Print strings raw without quotes */
        int len = strlen(json_result);
        if (len >= 2) {
            printf("< ");
            fwrite(json_result + 1, 1, len - 2, stdout);
            printf("\n");
        }
    } else {
        printf("< %s\n", json_result);
    }
}

static bool dispatch_special_command(ReplState *s, const char *cmd) {
    if (strcmp(cmd, ".exit") == 0) {
        alloy_engine_destroy(s->rt);
        exit(0);
    }
    if (strcmp(cmd, ".help") == 0) {
        printf(".help  Show this help\n.exit  Exit REPL\n.web   Show web process status\n.clear Clear multiline buffer\n");
        return true;
    }
    if (strcmp(cmd, ".clear") == 0) {
        s->multiline_len = 0;
        s->multiline_buf[0] = '\0';
        printf("Buffer cleared\n");
        return true;
    }
    if (strcmp(cmd, ".web") == 0) {
        /* Requirement 6.3: Print web process status */
        printf("web: not started (stub)\n");
        return true;
    }
    if (cmd[0] == '.') {
        fprintf(stderr, "unknown command: %s\n", cmd);
        return true;
    }
    return false;
}

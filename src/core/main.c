#include "alloy/alloy.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* Task 15: Main Entry Point */

extern int alloy_repl_run(int argc, char **argv);

void show_help() {
    printf("AlloyScript Phase 1 CLI\n");
    printf("Usage: alloy [options] [command|file]\n");
    printf("Commands:\n");
    printf("  repl          Start the interactive REPL\n");
    printf("Options:\n");
    printf("  -e <script>   Evaluate script\n");
    printf("  -v            Show version\n");
    printf("  --no-web      Disable web APIs (REPL only)\n");
    printf("  --help        Show help\n");
}

int main(int argc, char **argv) {
    if (argc < 2) {
        show_help();
        return 1;
    }

    if (strcmp(argv[1], "-v") == 0) {
        printf("AlloyScript v1.0.0-phase1\n");
        return 0;
    }

    if (strcmp(argv[1], "--help") == 0) {
        show_help();
        return 0;
    }

    /* Requirement 8.1: Wire repl subcommand */
    if (strcmp(argv[1], "repl") == 0) {
        return alloy_repl_run(argc - 1, argv + 1);
    }

    alloy_runtime_t *rt = alloy_engine_create();
    if (!rt) {
        fprintf(stderr, "Failed to create AlloyScript runtime\n");
        return 1;
    }

    if (strcmp(argv[1], "-e") == 0 && argc > 2) {
        JSValue res = alloy_secure_eval(rt, argv[2]);
        if (JS_IsException(res)) {
            fprintf(stderr, "Evaluation failed\n");
        }
        JS_FreeValue(rt->mqjs_ctx, res);
    } else {
        /* Execute file */
        FILE *f = fopen(argv[1], "r");
        if (f) {
            fseek(f, 0, SEEK_END); long size = ftell(f); fseek(f, 0, SEEK_SET);
            char *buf = malloc(size + 1); fread(buf, 1, size, f); buf[size] = 0;
            fclose(f);
            JSValue res = alloy_secure_eval(rt, buf);
            free(buf);
            JS_FreeValue(rt->mqjs_ctx, res);
        } else {
            fprintf(stderr, "Could not open file: %s\n", argv[1]);
        }
    }

    alloy_engine_destroy(rt);
    return 0;
}

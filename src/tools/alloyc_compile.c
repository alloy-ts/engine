#include "../../packages/mquickjs/mquickjs.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(int argc, char **argv) {
    // Parse -i <input>, -o <output>
    const char *input = NULL;
    const char *output = NULL;
    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "-i") == 0) input = argv[++i];
        if (strcmp(argv[i], "-o") == 0) output = argv[++i];
    }

    if (!input || !output) return 1;

    // Use JS_NewContext2 with prepare_compilation=true
    // JSContext *ctx = JS_NewContext2(NULL, 16<<20, &js_stdlib, 1);

    // JS_Eval with no RETVAL
    // JS_PrepareBytecode ...

    // Ed25519 signing simulation
    printf("[alloyc_compile] Compiling %s to %s\n", input, output);

    FILE *f = fopen(output, "wb");
    const char *magic = "ALLY";
    fwrite(magic, 1, 4, f);
    // Write version, key, sig, payload...
    fclose(f);

    return 0;
}

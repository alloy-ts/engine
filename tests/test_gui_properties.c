#include "../include/alloy/runtime.h"
#include <stdio.h>
#include <assert.h>
#include <string.h>
#include <stdlib.h>

void prop_all_ui_functions_present() {
    AlloyRuntime *rt = alloy_engine_create();
    const char *fns[] = {"createWindow", "closeWindow", "addControl", "setLayout", "requestCapability", "onEvent"};
    for (int i = 0; i < 6; i++) {
        char script[128];
        snprintf(script, sizeof(script), "typeof alloy.ui.%s", fns[i]);
        char *res = alloy_runtime_eval(rt, script, "prop.js");
        assert(strcmp(res, "\"function\"") == 0);
        free(res);
    }
    printf("prop_all_ui_functions_present passed\n");
    alloy_engine_destroy_runtime(rt);
}

int main() {
    prop_all_ui_functions_present();
    return 0;
}

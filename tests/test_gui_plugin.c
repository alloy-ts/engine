#include "../include/alloy/runtime.h"
#include "../include/alloy/gui.h"
#include <stdio.h>
#include <assert.h>
#include <string.h>
#include <stdlib.h>

void test_gui_plugin_registration() {
    AlloyRuntime *rt = alloy_engine_create();
    const RuntimePlugin *p = alloy_engine_find_plugin(rt, "alloy:gui");
    assert(p != NULL);
    printf("test_gui_plugin_registration passed\n");
    alloy_engine_destroy_runtime(rt);
}

void test_gui_symbols_present() {
    AlloyRuntime *rt = alloy_engine_create();
    char *res = alloy_runtime_eval(rt, "typeof alloy.ui.createWindow", "test.js");
    assert(strcmp(res, "\"function\"") == 0);
    free(res);
    printf("test_gui_symbols_present passed\n");
    alloy_engine_destroy_runtime(rt);
}

void test_gui_create_window_cap_enforcement() {
    AlloyRuntimeConfig cfg = alloy_config_defaults();
    AlloyRuntime *rt = alloy_engine_create_with_config(cfg);

    /* 1. Should fail with invalid cap_id */
    char *res = alloy_runtime_eval(rt, "JSON.stringify(alloy.ui.createWindow({title:'Test',width:800,height:600}, 'invalid_cap'))", "test.js");
    assert(strstr(res, "capability_denied") != NULL);
    free(res);

    /* 2. Should pass with valid cap_id */
    char *cap_res = alloy_runtime_eval(rt, "alloy.ui.requestCapability('ui:window', ['create'])", "test.js");
    /* cap_res is "cap_..." */
    char script[256];
    snprintf(script, sizeof(script), "alloy.ui.createWindow({title:'Test',width:800,height:600}, %s)", cap_res);
    char *win_res = alloy_runtime_eval(rt, script, "test.js");
    assert(atoi(win_res) > 0);
    free(cap_res);
    free(win_res);

    printf("test_gui_create_window_cap_enforcement passed\n");
    alloy_engine_destroy_runtime(rt);
}

void test_gui_invalid_dimensions() {
    AlloyRuntime *rt = alloy_engine_create();
    char *cap_res = alloy_runtime_eval(rt, "alloy.ui.requestCapability('ui:window', ['create'])", "test.js");

    char script[256];
    snprintf(script, sizeof(script), "JSON.stringify(alloy.ui.createWindow({title:'Test',width:0,height:600}, %s))", cap_res);
    char *res = alloy_runtime_eval(rt, script, "test.js");
    assert(strstr(res, "invalid_dimensions") != NULL);

    free(cap_res);
    free(res);
    printf("test_gui_invalid_dimensions passed\n");
    alloy_engine_destroy_runtime(rt);
}

int main() {
    test_gui_plugin_registration();
    test_gui_symbols_present();
    test_gui_create_window_cap_enforcement();
    test_gui_invalid_dimensions();
    return 0;
}

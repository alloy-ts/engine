#include "../../packages/webview/core/include/webview/webview.h"
#include <stdlib.h>
#include <stdio.h>

webview_t webview_create(int debug, void *window) {
    return (webview_t)malloc(1);
}

webview_error_t webview_destroy(webview_t w) {
    free(w);
    return WEBVIEW_ERROR_OK;
}

webview_error_t webview_run(webview_t w) {
    return WEBVIEW_ERROR_OK;
}

webview_error_t webview_eval(webview_t w, const char *js) {
    return WEBVIEW_ERROR_OK;
}

webview_error_t webview_bind(webview_t w, const char *name,
                             void (*fn)(const char *id, const char *req, void *arg),
                             void *arg) {
    return WEBVIEW_ERROR_OK;
}

webview_error_t webview_return(webview_t w, const char *id, int status, const char *result) {
    return WEBVIEW_ERROR_OK;
}

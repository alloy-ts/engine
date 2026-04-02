void alloy_webview_bind_global(webview_t w, const char *name, void (*fn)(const char *id, const char *req, void *arg), void *arg) {
    if (!w || !name) {
        fprintf(stderr, "[alloy] bind_global: NULL argument\n");
        return;
    }
    webview_bind(w, name, fn, arg);
}

#include "webview.h"
void* alloy_webview_init() { webview_t w = webview_create(0, NULL); webview_set_size(w, 0, 0, 0); return w; }

#ifndef ALLOYSCRIPT_BROWSER_API_TABLE_H
#define ALLOYSCRIPT_BROWSER_API_TABLE_H

typedef struct {
    const char *js_name;
    const char *webview_name;
} AlloyBrowserAPIEntry;

static const AlloyBrowserAPIEntry ALLOY_BROWSER_API_TABLE[] = {
    {"fetch", "window.fetch"},
    {"crypto", "window.crypto"},
    {"URL", "window.URL"},
    {"TextEncoder", "window.TextEncoder"},
    {"TextDecoder", "window.TextDecoder"},
    {"performance", "window.performance"},
    {NULL, NULL}
};

#endif

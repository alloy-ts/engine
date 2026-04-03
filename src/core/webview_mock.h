#ifndef ALLOYSCRIPT_WEBVIEW_MOCK_H
#define ALLOYSCRIPT_WEBVIEW_MOCK_H

typedef void* webview_t;
typedef int webview_error_t;

typedef enum {
  WEBVIEW_HINT_NONE,
  WEBVIEW_HINT_MIN,
  WEBVIEW_HINT_MAX,
  WEBVIEW_HINT_FIXED,
} webview_hint_t;

static inline webview_t webview_create(int debug, void *window) { return (webview_t)1; }
static inline webview_error_t webview_destroy(webview_t w) { return 0; }
static inline webview_error_t webview_run(webview_t w) { return 0; }
static inline webview_error_t webview_terminate(webview_t w) { return 0; }
static inline webview_error_t webview_set_size(webview_t w, int width, int height, webview_hint_t hints) { return 0; }
static inline webview_error_t webview_set_title(webview_t w, const char *title) { return 0; }
static inline webview_error_t webview_navigate(webview_t w, const char *url) { return 0; }
static inline webview_error_t webview_eval(webview_t w, const char *js) { return 0; }
static inline webview_error_t webview_bind(webview_t w, const char *name, void (*fn)(const char *id, const char *req, void *arg), void *arg) { return 0; }
static inline webview_error_t webview_return(webview_t w, const char *id, int status, const char *result) { return 0; }

#endif

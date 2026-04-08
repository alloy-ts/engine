#include "../../include/alloy/gui.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

typedef struct {
    int id;
    void *handle;
} MockEntry;

typedef struct {
    MockEntry windows[16];
    int window_count;
    MockEntry controls[256];
    int control_count;
} MockBackendState;

static void *mock_create_window(AlloyWindowOptions *opts) {
    (void)opts;
    static int window_id_gen = 1;
    return (void *)(uintptr_t)(window_id_gen++);
}

static void mock_destroy_window(void *handle) {
    (void)handle;
}

static void mock_show_window(void *handle) {
    (void)handle;
}

static void mock_hide_window(void *handle) {
    (void)handle;
}

static void mock_set_title(void *handle, const char *title) {
    (void)handle; (void)title;
}

static void mock_set_frame(void *handle, int x, int y, int w, int h) {
    (void)handle; (void)x; (void)y; (void)w; (void)h;
}

static void *mock_create_control(void *window_handle, AlloyControlSpec *spec) {
    (void)window_handle; (void)spec;
    static int control_id_gen = 1;
    return (void *)(uintptr_t)(control_id_gen++);
}

static void mock_destroy_control(void *control_handle) {
    (void)control_handle;
}

static void mock_set_control_frame(void *control_handle, int x, int y, int w, int h) {
    (void)control_handle; (void)x; (void)y; (void)w; (void)h;
}

static void mock_set_control_prop(void *control_handle, const char *key, const char *value_json) {
    (void)control_handle; (void)key; (void)value_json;
}

static void *mock_create_webview(void *window_handle, int x, int y, int w, int h) {
    (void)window_handle; (void)x; (void)y; (void)w; (void)h;
    static int wv_id_gen = 1001;
    return (void *)(uintptr_t)(wv_id_gen++);
}

static void mock_webview_load_url(void *wv_handle, const char *url) {
    (void)wv_handle; (void)url;
}

static void mock_webview_load_html(void *wv_handle, const char *html) {
    (void)wv_handle; (void)html;
}

static int mock_set_menu_bar(const char *template_json) {
    (void)template_json;
    return 0;
}

static int mock_set_tray_icon(const char *icon_path, const char *tooltip) {
    (void)icon_path; (void)tooltip;
    return 0;
}

AlloyGuiBackend *alloy_gui_backend_create_mock(void) {
    AlloyGuiBackend *b = calloc(1, sizeof(AlloyGuiBackend));
    b->create_window = mock_create_window;
    b->destroy_window = mock_destroy_window;
    b->show_window = mock_show_window;
    b->hide_window = mock_hide_window;
    b->set_title = mock_set_title;
    b->set_frame = mock_set_frame;
    b->create_control = mock_create_control;
    b->destroy_control = mock_destroy_control;
    b->set_control_frame = mock_set_control_frame;
    b->set_control_prop = mock_set_control_prop;
    b->create_webview = mock_create_webview;
    b->webview_load_url = mock_webview_load_url;
    b->webview_load_html = mock_webview_load_html;
    b->set_menu_bar = mock_set_menu_bar;
    b->set_tray_icon = mock_set_tray_icon;
    return b;
}

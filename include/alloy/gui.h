#ifndef ALLOY_GUI_H
#define ALLOY_GUI_H

#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Permission bits for ui:* capabilities */
typedef enum {
    ALLOY_UI_PERM_WINDOW_CREATE   = (1 << 4),
    ALLOY_UI_PERM_WINDOW_CLOSE    = (1 << 5),
    ALLOY_UI_PERM_CONTROL_CREATE  = (1 << 6),
    ALLOY_UI_PERM_CONTROL_MODIFY  = (1 << 7),
    ALLOY_UI_PERM_NAVIGATE        = (1 << 8),
} AlloyUiPermission;

/* Window creation options (mirrors JSX <Window> props) */
typedef struct {
    const char *title;
    int         width, height, x, y;
    bool        resizable, minimizable, maximizable;
    bool        closable, fullscreen, always_on_top, transparent;
    bool        isolated;          /* spawn alloy_renderer child */
    const char *capabilities_json; /* JSON array of cap_ids for isolated window */
} AlloyWindowOptions;

/* Control spec (mirrors JSX element props, serialised as JSON) */
typedef struct {
    const char *type;        /* "Button", "Label", "TextField", etc. */
    const char *props_json;  /* full props as JSON object */
    int         parent_id;   /* 0 = window root, >0 = parent control */
} AlloyControlSpec;

/*
 * AlloyGuiBackend — platform abstraction vtable for alloy_gui process.
 * All pointers nullable; gui_process.c returns {"error":"not_supported"} for NULL.
 */
typedef struct AlloyGuiBackend {
    void *(*create_window)  (AlloyWindowOptions *opts);
    void  (*destroy_window) (void *handle);
    void  (*show_window)    (void *handle);
    void  (*hide_window)    (void *handle);
    void  (*set_title)      (void *handle, const char *title);
    void  (*set_frame)      (void *handle, int x, int y, int w, int h);

    /* Native controls */
    void *(*create_control) (void *window_handle, AlloyControlSpec *spec);
    void  (*destroy_control)(void *control_handle);
    void  (*set_control_frame)(void *control_handle, int x, int y, int w, int h);
    void  (*set_control_prop)(void *control_handle, const char *key, const char *value_json);

    /* WebView control (explicit only) */
    void *(*create_webview) (void *window_handle, int x, int y, int w, int h);
    void  (*webview_load_url)(void *wv_handle, const char *url);
    void  (*webview_load_html)(void *wv_handle, const char *html);

    /* Native chrome */
    int   (*set_menu_bar)   (const char *template_json);
    int   (*set_tray_icon)  (const char *icon_path, const char *tooltip);
} AlloyGuiBackend;

AlloyGuiBackend *alloy_gui_backend_create(void);
void             alloy_gui_backend_destroy(AlloyGuiBackend *b);

/* Mock backend */
AlloyGuiBackend *alloy_gui_backend_create_mock(void);

#ifdef __cplusplus
}
#endif

#endif /* ALLOY_GUI_H */

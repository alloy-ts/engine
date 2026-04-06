/**
 * AlloyScript — Native GUI plugin public interface
 *
 * Defines the AlloyGuiBackend vtable (platform abstraction used inside
 * alloy_gui process), the AlloyUiPermission capability bits, and the
 * window / control option types shared across the IPC boundary.
 *
 * This header is included by:
 *   - src/gui/gui_plugin.c          (alloy_host — capability enforcement)
 *   - src/gui/alloy_gui_main.c      (alloy_gui process — backend dispatch)
 *   - src/gui/backends/*.c / *.m    (platform implementations)
 *   - tests/test_gui_plugin.c
 *   - tests/test_gui_properties.c
 */
#ifndef ALLOY_GUI_H
#define ALLOY_GUI_H

#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ── Capability permission bits for ui:* resources ─────────────────────── */
/**
 * AlloyUiPermission — OR'd into the uint32_t permissions field of AlloyCap.
 * Occupies bits 4–8 to avoid colliding with base AlloyPermission bits 0–3.
 */
typedef enum {
    ALLOY_UI_PERM_WINDOW_CREATE   = (1 << 4),  /**< create native windows     */
    ALLOY_UI_PERM_WINDOW_CLOSE    = (1 << 5),  /**< close existing windows    */
    ALLOY_UI_PERM_CONTROL_CREATE  = (1 << 6),  /**< add controls to windows   */
    ALLOY_UI_PERM_CONTROL_MODIFY  = (1 << 7),  /**< set_layout / set_prop     */
    ALLOY_UI_PERM_NAVIGATE        = (1 << 8),  /**< webview_load_url          */
} AlloyUiPermission;

/* ── Window creation options ────────────────────────────────────────────── */
/**
 * AlloyWindowOptions — mirrors the <Window> JSX props after macro expansion.
 * All string pointers must remain valid for the lifetime of the IPC call.
 */
typedef struct {
    const char *title;
    int         width, height, x, y;
    bool        resizable;
    bool        minimizable;
    bool        maximizable;
    bool        closable;
    bool        fullscreen;
    bool        always_on_top;
    bool        transparent;

    /**
     * isolated — spawn alloy_renderer child for this window.
     * capabilities_json — JSON array of cap_id strings forwarded to renderer.
     */
    bool        isolated;
    const char *capabilities_json;
} AlloyWindowOptions;

/* ── Control specification ──────────────────────────────────────────────── */
/**
 * AlloyControlSpec — serialised representation of a JSX element after macro
 * expansion.  props_json carries all element-specific attributes.
 *
 * Supported types (mirrors Requirement 7.1):
 *   "Button" "Label" "TextField" "TextArea" "Checkbox" "Radio"
 *   "ComboBox" "List" "Menu" "Toolbar" "StatusBar" "Panel" "Spacer"
 *   "WebView"  (explicit only — never created implicitly)
 *   "Stack" "Flex" "ScrollView" "Grid"  (layout containers)
 */
typedef struct {
    const char *type;        /**< element type string (see above)          */
    const char *props_json;  /**< full props as JSON object                */
    int         parent_id;   /**< 0 = window root, >0 = parent control_id */
} AlloyControlSpec;

/* ── Platform backend vtable ────────────────────────────────────────────── */
/**
 * AlloyGuiBackend — implemented once per platform (Cocoa, Win32, mock).
 *
 * Lives inside the alloy_gui process only.  alloy_host never includes or
 * links against the platform backend; it only talks over GUI_IPC.
 *
 * All function pointers are NULLABLE.  gui_plugin.c returns
 * {"error":"not_supported"} for any NULL pointer without crashing.
 *
 * Ownership:
 *   - handle values returned by create_* are opaque platform pointers owned
 *     by the backend; they are freed via the corresponding destroy_* call.
 *   - String arguments (title, url, icon_path, …) are borrowed — the backend
 *     must copy them if it needs them beyond the call.
 */
typedef struct AlloyGuiBackend {
    /* ── Window operations ─────────────────────────────────────────────── */
    /** Create a native window; return opaque platform handle or NULL. */
    void *(*create_window)  (AlloyWindowOptions *opts);
    /** Destroy a window and all its children. */
    void  (*destroy_window) (void *handle);
    /** Show a previously created (possibly hidden) window. */
    void  (*show_window)    (void *handle);
    /** Hide a window without destroying it. */
    void  (*hide_window)    (void *handle);
    /** Update the window title bar text. */
    void  (*set_title)      (void *handle, const char *title);
    /** Resize and/or move the window to (x, y, w, h). */
    void  (*set_frame)      (void *handle, int x, int y, int w, int h);

    /* ── Native controls ────────────────────────────────────────────────── */
    /** Create a native control inside window_handle; return control handle. */
    void *(*create_control)      (void *window_handle, AlloyControlSpec *spec);
    /** Destroy a single control. */
    void  (*destroy_control)     (void *control_handle);
    /** Move and resize an existing control (called by LayoutEngine). */
    void  (*set_control_frame)   (void *control_handle, int x, int y, int w, int h);
    /** Set a single named prop on a control (value is JSON-encoded). */
    void  (*set_control_prop)    (void *control_handle,
                                  const char *key, const char *value_json);

    /* ── WebView control (explicit only) ────────────────────────────────── */
    /**
     * Create an embedded WebView inside window_handle at the given frame.
     * ONLY called when the user explicitly writes <WebView> in JSX.
     * Return an opaque WebView handle or NULL.
     */
    void *(*create_webview)      (void *window_handle, int x, int y, int w, int h);
    /** Navigate the WebView to a URL. */
    void  (*webview_load_url)    (void *wv_handle, const char *url);
    /** Load raw HTML into the WebView. */
    void  (*webview_load_html)   (void *wv_handle, const char *html);

    /* ── Native chrome ──────────────────────────────────────────────────── */
    /**
     * Replace the application menu bar from a JSON template.
     * Returns 0 on success, negative on error.
     */
    int   (*set_menu_bar)        (const char *template_json);
    /**
     * Set the system tray icon.
     * icon_path — absolute path to an image file (PNG/ICO).
     * tooltip   — text shown on hover.
     * Returns 0 on success, negative on error.
     */
    int   (*set_tray_icon)       (const char *icon_path, const char *tooltip);
} AlloyGuiBackend;

/* ── Backend lifecycle ──────────────────────────────────────────────────── */
/**
 * alloy_gui_backend_create — allocate and initialise the platform backend.
 * On macOS: returns the Cocoa backend.
 * On Windows: returns the Win32 backend.
 * In test builds: returns the mock backend (no platform calls).
 */
AlloyGuiBackend *alloy_gui_backend_create(void);

/**
 * alloy_gui_backend_create_mock — always returns the in-memory mock backend.
 * Exported for use in unit and property-based tests.
 */
AlloyGuiBackend *alloy_gui_backend_create_mock(void);

/** Free the backend and any associated resources. */
void             alloy_gui_backend_destroy(AlloyGuiBackend *b);

#ifdef __cplusplus
}
#endif

#endif /* ALLOY_GUI_H */

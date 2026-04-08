#include <windows.h>
#include "../../include/alloy/gui.h"

static LRESULT CALLBACK WindowProc(HWND hwnd, UINT uMsg, WPARAM wParam, LPARAM lParam) {
    switch (uMsg) {
        case WM_CLOSE:
            /* Send ALLOY_MSG_GUI_EVENT {type:"closed"} */
            DestroyWindow(hwnd);
            return 0;
        case WM_DESTROY:
            PostQuitMessage(0);
            return 0;
    }
    return DefWindowProc(hwnd, uMsg, wParam, lParam);
}

static void *win32_create_window(AlloyWindowOptions *opts) {
    HINSTANCE hInstance = GetModuleHandle(NULL);
    const char *CLASS_NAME = "AlloyWindow";

    WNDCLASS wc = {0};
    wc.lpfnWndProc = WindowProc;
    wc.hInstance = hInstance;
    wc.lpszClassName = CLASS_NAME;
    RegisterClass(&wc);

    HWND hwnd = CreateWindowEx(
        0, CLASS_NAME, opts->title, WS_OVERLAPPEDWINDOW,
        CW_USEDEFAULT, CW_USEDEFAULT, opts->width, opts->height,
        NULL, NULL, hInstance, NULL
    );

    if (hwnd) ShowWindow(hwnd, SW_SHOW);
    return (void *)hwnd;
}

static void win32_destroy_window(void *handle) {
    DestroyWindow((HWND)handle);
}

/* ... implement other win32 backend functions ... */

AlloyGuiBackend *alloy_gui_backend_create(void) {
    AlloyGuiBackend *b = calloc(1, sizeof(AlloyGuiBackend));
    b->create_window = win32_create_window;
    b->destroy_window = win32_destroy_window;
    /* ... */
    return b;
}

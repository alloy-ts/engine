#include <windows.h>
#include "gui.h"

/* Requirement 11: Windows Win32 Backend */

void *win32_create_window(AlloyWindowOptions *opts) {
    HWND hwnd = CreateWindowEx(0, "AlloyWindowClass", opts->title, WS_OVERLAPPEDWINDOW,
                               CW_USEDEFAULT, CW_USEDEFAULT, opts->width, opts->height,
                               NULL, NULL, GetModuleHandle(NULL), NULL);
    ShowWindow(hwnd, SW_SHOW);
    return (void *)hwnd;
}

AlloyGuiBackend *alloy_gui_backend_create_win32(void) {
    AlloyGuiBackend *b = calloc(1, sizeof(AlloyGuiBackend));
    b->create_window = win32_create_window;
    /* Other implementations... */
    return b;
}

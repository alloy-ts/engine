#include "gui.h"
#include "../capability.h"
#include <stdlib.h>
#include <stdio.h>

struct NativeWindow {
    WindowOptions opts;
    void *platform_handle;
};

NativeWindow* alloy_gui_window_create(const char *cap_token, WindowOptions *opts) {
    if (!alloy_capability_validate(cap_token, CAP_GUI_WINDOW, opts->title)) {
        return NULL;
    }
    NativeWindow *win = malloc(sizeof(NativeWindow));
    win->opts = *opts;
    return win;
}

void alloy_gui_window_show(NativeWindow *win) {}
void alloy_gui_window_close(NativeWindow *win) { free(win); }
void alloy_gui_window_set_title(NativeWindow *win, const char *title) {}
void alloy_gui_poll_events(void) {}

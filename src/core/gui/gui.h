#ifndef ALLOY_GUI_H
#define ALLOY_GUI_H

#include <stdint.h>
#include <stdbool.h>

typedef struct NativeWindow NativeWindow;

typedef struct {
    int width;
    int height;
    const char *title;
    bool resizable;
    bool borderless;
} WindowOptions;

NativeWindow* alloy_gui_window_create(const char *cap_token, WindowOptions *opts);
void alloy_gui_window_show(NativeWindow *win);
void alloy_gui_window_close(NativeWindow *win);
void alloy_gui_window_set_title(NativeWindow *win, const char *title);
void alloy_gui_poll_events(void);

#endif

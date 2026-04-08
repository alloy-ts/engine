#include "gui.h"
#include <stdlib.h>
#include <stdio.h>

/* Requirement 3.1: Mock backend implementation */

static void *mock_create_window(AlloyWindowOptions *opts) {
    printf("[mock] Creating window: %s (%dx%d)\n", opts->title, opts->width, opts->height);
    return (void *)0x1234; // Dummy handle
}

static void mock_destroy_window(void *handle) {
    printf("[mock] Destroying window: %p\n", handle);
}

static void mock_show_window(void *handle) {
    printf("[mock] Showing window: %p\n", handle);
}

static void mock_hide_window(void *handle) {
    printf("[mock] Hiding window: %p\n", handle);
}

static void mock_set_title(void *handle, const char *title) {
    printf("[mock] Setting title for %p: %s\n", handle, title);
}

static void *mock_create_control(void *window_handle, AlloyControlSpec *spec) {
    printf("[mock] Creating control: %s for window %p\n", spec->type, window_handle);
    return (void *)0x5678; // Dummy handle
}

AlloyGuiBackend *alloy_gui_backend_create_mock(void) {
    AlloyGuiBackend *b = calloc(1, sizeof(AlloyGuiBackend));
    b->create_window = mock_create_window;
    b->destroy_window = mock_destroy_window;
    b->show_window = mock_show_window;
    b->hide_window = mock_hide_window;
    b->set_title = mock_set_title;
    b->create_control = mock_create_control;
    return b;
}

void alloy_gui_backend_destroy(AlloyGuiBackend *b) {
    free(b);
}

#include "alloy/gui.h"
#include <stdlib.h>
#include <string.h>

typedef struct {
    void *handle;
} MockWindow;

static void *mock_create_window(AlloyWindowOptions *opts) {
    MockWindow *w = malloc(sizeof(MockWindow));
    w->handle = (void *)0x1234;
    return w;
}

static void mock_destroy_window(void *handle) {
    free(handle);
}

AlloyGuiBackend *alloy_gui_backend_create_mock(void) {
    AlloyGuiBackend *b = calloc(1, sizeof(AlloyGuiBackend));
    b->create_window = mock_create_window;
    b->destroy_window = mock_destroy_window;
    return b;
}

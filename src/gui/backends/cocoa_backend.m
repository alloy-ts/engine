#import <Cocoa/Cocoa.h>
#include "gui.h"

/* Requirement 10: macOS Cocoa Backend */

void *cocoa_create_window(AlloyWindowOptions *opts) {
    NSRect rect = NSMakeRect(opts->x, opts->y, opts->width, opts->height);
    NSWindow *win = [[NSWindow alloc] initWithContentRect:rect
                                              styleMask:NSWindowStyleMaskTitled
                                                backing:NSBackingStoreBuffered
                                                  defer:NO];
    [win setTitle:[NSString stringWithUTF8String:opts->title]];
    [win makeKeyAndOrderFront:nil];
    return (void *)win;
}

AlloyGuiBackend *alloy_gui_backend_create(void) {
    AlloyGuiBackend *b = calloc(1, sizeof(AlloyGuiBackend));
    b->create_window = cocoa_create_window;
    /* Other implementations... */
    return b;
}

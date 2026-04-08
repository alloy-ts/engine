#import <Cocoa/Cocoa.h>
#import <WebKit/WebKit.h>
#include "../../include/alloy/gui.h"

@interface AlloyWindowDelegate : NSObject <NSWindowDelegate>
@property (nonatomic, assign) void *handle;
@end

@implementation AlloyWindowDelegate
- (void)windowWillClose:(NSNotification *)notification {
    /* Send ALLOY_MSG_GUI_EVENT {type:"closed"} */
}
@end

static void *cocoa_create_window(AlloyWindowOptions *opts) {
    NSRect rect = NSMakeRect(opts->x, opts->y, opts->width, opts->height);
    NSUInteger style = NSWindowStyleMaskTitled | NSWindowStyleMaskClosable;
    if (opts->resizable) style |= NSWindowStyleMaskResizable;
    if (opts->minimizable) style |= NSWindowStyleMaskMiniaturizable;

    NSWindow *window = [[NSWindow alloc] initWithContentRect:rect
                                                   styleMask:style
                                                     backing:NSBackingStoreBuffered
                                                       defer:NO];
    [window setTitle:[NSString stringWithUTF8String:opts->title]];
    [window makeKeyAndOrderFront:nil];

    AlloyWindowDelegate *delegate = [[AlloyWindowDelegate alloc] init];
    delegate.handle = (__bridge void *)window;
    [window setDelegate:delegate];

    return (__bridge_retained void *)window;
}

static void cocoa_destroy_window(void *handle) {
    NSWindow *window = (__bridge_transfer NSWindow *)handle;
    [window close];
}

/* ... implement other cocoa backend functions ... */

AlloyGuiBackend *alloy_gui_backend_create(void) {
    AlloyGuiBackend *b = calloc(1, sizeof(AlloyGuiBackend));
    b->create_window = cocoa_create_window;
    b->destroy_window = cocoa_destroy_window;
    /* ... */
    return b;
}

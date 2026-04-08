CC=gcc
CFLAGS=-Wall -g -I../../packages/mquickjs -I../../packages/webview/core/include -D_GNU_SOURCE
LDFLAGS=-L../../packages/mquickjs -lm -lcrypto -lssl -lpthread

MQJS_OBJS=../../packages/mquickjs/mquickjs.o \
          ../../packages/mquickjs/dtoa.o \
          ../../packages/mquickjs/libm.o \
          ../../packages/mquickjs/cutils.o

# WebView source (compiling it directly since we don't have a prebuilt lib)
WEBVIEW_SRC=../../packages/webview/core/src/webview.cc
# For headless/linux we might need specific flags, but let's try a simple approach
WEBVIEW_FLAGS=-DWEBVIEW_GTK=1 $(shell pkg-config --cflags --libs gtk+-3.0 webkit2gtk-4.0 2>/dev/null)

all: test_engine

alloyscript.o: alloyscript.c alloyscript.h
	$(CC) $(CFLAGS) -c -o $@ $<

test_engine: test_engine.o alloyscript.o $(MQJS_OBJS)
	$(CC) -o $@ $^ $(LDFLAGS)

test_engine.o: test_engine.c alloyscript.h
	$(CC) $(CFLAGS) -c -o $@ $<

GUI_OBJS=gui_plugin.o mock_backend.o
ALLOY_GUI_OBJS=alloy_gui_main.o layout_engine.o cocoa_backend.o win32_backend.o

# Requirement 1.1: ALLOY_ENABLE_GUI gating
ifeq ($(ALLOY_ENABLE_GUI), 1)
CFLAGS += -DALLOY_ENABLE_GUI=1
all: test_engine alloy_gui alloy_renderer
endif

alloy_gui: $(ALLOY_GUI_OBJS)
	$(CC) -o $@ $^ $(LDFLAGS)

alloy_renderer: alloy_renderer_main.o
	$(CC) -o $@ $^ $(LDFLAGS)

clean:
	rm -f *.o test_engine alloy_gui alloy_renderer

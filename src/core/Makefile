CC=gcc
CFLAGS=-Wall -g -I../../packages/mquickjs -I../../packages/webview/core/include -D_GNU_SOURCE
LDFLAGS=-L../../packages/mquickjs -lm -lcrypto -lssl

MQJS_OBJS=../../packages/mquickjs/mquickjs.o \
          ../../packages/mquickjs/dtoa.o \
          ../../packages/mquickjs/libm.o \
          ../../packages/mquickjs/cutils.o

# WebView source (compiling it directly since we don't have a prebuilt lib)
WEBVIEW_SRC=../../packages/webview/core/src/webview.cc
# For headless/linux we might need specific flags, but let's try a simple approach
WEBVIEW_FLAGS=-DWEBVIEW_GTK=1 $(shell pkg-config --cflags --libs gtk+-3.0 webkit2gtk-4.0 2>/dev/null)

all: engine.o ipc.o

engine.o: engine.c engine.h browser_api_table.h ipc.h js_engine.h
	$(CC) $(CFLAGS) -c -o $@ engine.c

ipc.o: ipc.c ipc.h engine.h
	$(CC) $(CFLAGS) -c -o $@ ipc.c

mqjs_bridge.o: backend_adapters/mqjs_bridge.c backend_adapters/mqjs_bridge.h js_engine.h
	$(CC) $(CFLAGS) -c -o $@ backend_adapters/mqjs_bridge.c

webviewjs_bridge.o: backend_adapters/webviewjs_bridge.c js_engine.h
	$(CC) $(CFLAGS) -c -o $@ backend_adapters/webviewjs_bridge.c

clean:
	rm -f *.o test_engine

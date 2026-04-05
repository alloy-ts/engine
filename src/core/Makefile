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

all: test_engine

engine.o: engine.c engine.h
	$(CC) $(CFLAGS) -I. -c -o $@ $<

ipc.o: ipc.c ipc.h
	$(CC) $(CFLAGS) -c -o $@ $<

mqjs_bridge.o: backend_adapters/mqjs_bridge.c backend_adapters/mqjs_bridge.h
	$(CC) $(CFLAGS) -c -o $@ $<

webviewjs_bridge.o: backend_adapters/webviewjs_bridge.c
	$(CC) $(CFLAGS) -c -o $@ $<

blake3.o: blake3/blake3.c blake3/blake3.h
	$(CC) $(CFLAGS) -c -o $@ $<

test_engine: test_engine.o engine.o ipc.o mqjs_bridge.o webviewjs_bridge.o blake3.o $(MQJS_OBJS)
	$(CC) -o $@ $^ $(LDFLAGS)

test_engine.o: test_engine.c engine.h
	$(CC) $(CFLAGS) -c -o $@ $<

clean:
	rm -f *.o test_engine

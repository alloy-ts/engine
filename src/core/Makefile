CC=gcc
CFLAGS=-Wall -g -I../../packages/mquickjs -I../../packages/webview/core/include -D_GNU_SOURCE -DMOCK_WEBVIEW
LDFLAGS=-L../../packages/mquickjs -lm -lcrypto -lssl

MQJS_OBJS=../../packages/mquickjs/mquickjs.o \
          ../../packages/mquickjs/dtoa.o \
          ../../packages/mquickjs/libm.o \
          ../../packages/mquickjs/cutils.o

CRYPTO_OBJS=crypto/blake3_stub.o

# WebView source (compiling it directly since we don't have a prebuilt lib)
WEBVIEW_SRC=../../packages/webview/core/src/webview.cc
# For headless/linux we might need specific flags, but let's try a simple approach
WEBVIEW_FLAGS=-DWEBVIEW_GTK=1 $(shell pkg-config --cflags --libs gtk+-3.0 webkit2gtk-4.0 2>/dev/null)

WEBVIEW_OBJ=../../packages/webview/core/src/webview.o

ADAPTER_OBJS=backend_adapters/mqjs_bridge.o backend_adapters/webviewjs_bridge.o backend_adapters/mqjs_wrapper.o

all: test_engine

engine.o: engine.c engine.h js_engine.h browser_api_table.h bridge.h
	$(CC) $(CFLAGS) -c -o $@ $<

backend_adapters/mqjs_bridge.o: backend_adapters/mqjs_bridge.c backend_adapters/mqjs_bridge.h js_engine.h
	$(CC) $(CFLAGS) -c -o $@ $<

backend_adapters/mqjs_wrapper.o: backend_adapters/mqjs_wrapper.c
	$(CC) $(CFLAGS) -c -o $@ $<

backend_adapters/webviewjs_bridge.o: backend_adapters/webviewjs_bridge.c backend_adapters/webviewjs_bridge.h js_engine.h
	$(CC) $(CFLAGS) -c -o $@ $<

ipc.o: ipc.c ipc.h engine.h
	$(CC) $(CFLAGS) -c -o $@ $<

test_engine: test_engine.o engine.o ipc.o $(ADAPTER_OBJS) $(MQJS_OBJS) $(CRYPTO_OBJS)
	$(CC) -o $@ $^ $(LDFLAGS)

test_engine.o: test_engine.c engine.h
	$(CC) $(CFLAGS) -c -o $@ $<

crypto/blake3_stub.o: crypto/blake3_stub.c crypto/blake3.h
	$(CC) $(CFLAGS) -c -o $@ $<

clean:
	rm -f *.o backend_adapters/*.o crypto/*.o test_engine

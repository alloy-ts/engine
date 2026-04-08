CC=gcc
CFLAGS=-Wall -g -I../../packages/mquickjs -I../../packages/webview/core/include -D_GNU_SOURCE
LDFLAGS=-L../../packages/mquickjs -lm -lcrypto -lssl

MQJS_OBJS=../../packages/mquickjs/mquickjs.o \
          ../../packages/mquickjs/dtoa.o \
          ../../packages/mquickjs/libm.o \
          ../../packages/mquickjs/cutils.o

# WebView source (mocking for headless tests in sandbox)
WEBVIEW_MOCK_SRC=backend_adapters/webview_mock.c

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

os_process.o: os_process.c os_process.h
	$(CC) $(CFLAGS) -c -o $@ $<

capability.o: capability.c capability.h
	$(CC) $(CFLAGS) -c -o $@ $<

wasi_filesystem.o: wasi/wasi_filesystem.c wasi/wasi_filesystem.h
	$(CC) $(CFLAGS) -c -o $@ $<

wasi_http.o: wasi/wasi_http.c wasi/wasi_http.h
	$(CC) $(CFLAGS) -c -o $@ $<

gui_common.o: gui/gui_common.c gui/gui.h
	$(CC) $(CFLAGS) -c -o $@ $<

webview_mock.o: backend_adapters/webview_mock.c
	$(CC) $(CFLAGS) -c -o $@ $<

test_engine: test_engine.o engine.o ipc.o mqjs_bridge.o webviewjs_bridge.o blake3.o os_process.o capability.o wasi_filesystem.o wasi_http.o gui_common.o webview_mock.o $(MQJS_OBJS)
	$(CC) -o $@ $^ $(LDFLAGS)

test_engine.o: test_engine.c engine.h
	$(CC) $(CFLAGS) -c -o $@ $<

clean:
	rm -f *.o test_engine

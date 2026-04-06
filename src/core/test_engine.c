#include "alloyscript.h"
#include <stdio.h>
#include <string.h>

int main() {
    alloy_runtime_t *rt = alloy_init(1024 * 1024 * 4);
    if (!rt) {
        fprintf(stderr, "Failed to initialise AlloyScript runtime\n");
        return 1;
    }

    printf("--- AlloyScript Runtime API Tests ---\n");

    const char *test_script =
        "(function() {"
        "  try {"
        "    console.log('Testing ReadableStream...');"
        "    var stream = new ReadableStream({"
        "      start: function(controller) {"
        "        controller.enqueue('hello');"
        "        controller.enqueue('world');"
        "        controller.close();"
        "      }"
        "    });"
        "    Alloy.readableStreamToText(stream).then(function(text) {"
        "       console.log('Stream result:', text);"
        "    });"
        ""
        "    console.log('Testing Buffer...');"
        "    var buf = Buffer.from('Alloy');"
        "    console.log('Buffer to hex:', buf.toString('hex'));"
        ""
        "    console.log('Testing ArrayBufferSink...');"
        "    var sink = new Alloy.ArrayBufferSink();"
        "    sink.start({ asUint8Array: true });"
        "    sink.write('A');"
        "    sink.write('B');"
        "    console.log('Sink result:', new TextDecoder().decode(sink.end()));"
        ""
        "    console.log('Testing AlloyFile...');"
        "    var file = Alloy.file('runtime.js');"
        "    console.log('File name:', file.name);"
        "    console.log('File size:', file.size);"
        ""
        "    console.log('ES5 tests completed.');"
        "  } catch (e) {"
        "    console.log('Test failed:', e.toString());"
        "  }"
        "})();";

    JSValue res = alloy_secure_eval(rt, test_script);
    if (JS_IsException(res)) {
        JSValue err = JS_GetException(rt->mqjs_ctx);
        JSCStringBuf buf;
        printf("Eval exception: %s\n", JS_ToCString(rt->mqjs_ctx, err, &buf));
    }

    alloy_destroy(rt);
    return 0;
}

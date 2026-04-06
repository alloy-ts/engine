#ifndef ALLOYSCRIPT_BROWSER_API_TABLE_H
#define ALLOYSCRIPT_BROWSER_API_TABLE_H

#include <stddef.h>

typedef enum {
    ARGS_NONE = 0,
    ARGS_FETCH = 1,
    ARGS_CRYPTO_RANDOM = 2,
    ARGS_CRYPTO_SUBTLE = 3,
    ARGS_URL = 4,
    ARGS_BLOB = 5,
    ARGS_STREAM = 6,
    ARGS_EVENT = 7,
    ARGS_STORAGE = 8,
    ARGS_WORKER = 9,
    ARGS_WEBSOCKET = 10,
    ARGS_ENCODING = 11,
    ARGS_OBJECT = 12,
    ARGS_STRING = 13,
} AlloyArgSchema;

typedef enum {
    RET_NONE = 0,
    RET_RESPONSE = 1,
    RET_OBJECT = 2,
    RET_STRING = 3,
    RET_NUMBER = 4,
    RET_BOOLEAN = 5,
    RET_ARRAYBUFFER = 6,
    RET_PROMISE = 7,
} AlloyReturnSchema;

typedef struct {
    const char *js_name;
    const char *webview_method;
    AlloyArgSchema arg_schema;
    AlloyReturnSchema return_schema;
} AlloyBrowserAPIEntry;

static const AlloyBrowserAPIEntry ALLOY_BROWSER_API_TABLE[] = {
    /* HTTP */
    {"fetch", "fetch", ARGS_FETCH, RET_PROMISE},
    {"Request", "Request", ARGS_OBJECT, RET_OBJECT},
    {"Response", "Response", ARGS_OBJECT, RET_OBJECT},
    {"Headers", "Headers", ARGS_OBJECT, RET_OBJECT},
    {"AbortController", "AbortController", ARGS_NONE, RET_OBJECT},
    {"AbortSignal", "AbortSignal", ARGS_NONE, RET_OBJECT},

    /* URLs */
    {"URL", "URL", ARGS_URL, RET_OBJECT},
    {"URLSearchParams", "URLSearchParams", ARGS_URL, RET_OBJECT},

    /* Web Workers */
    {"Worker", "Worker", ARGS_WORKER, RET_OBJECT},
    {"MessagePort", "MessagePort", ARGS_NONE, RET_OBJECT},
    {"MessageChannel", "MessageChannel", ARGS_NONE, RET_OBJECT},
    {"BroadcastChannel", "BroadcastChannel", ARGS_STRING, RET_OBJECT},

    /* Streams */
    {"ReadableStream", "ReadableStream", ARGS_STREAM, RET_OBJECT},
    {"WritableStream", "WritableStream", ARGS_STREAM, RET_OBJECT},
    {"TransformStream", "TransformStream", ARGS_STREAM, RET_OBJECT},
    {"ByteLengthQueuingStrategy", "ByteLengthQueuingStrategy", ARGS_OBJECT, RET_OBJECT},
    {"CountQueuingStrategy", "CountQueuingStrategy", ARGS_OBJECT, RET_OBJECT},

    /* Blob */
    {"Blob", "Blob", ARGS_BLOB, RET_OBJECT},
    {"File", "File", ARGS_BLOB, RET_OBJECT},
    {"FormData", "FormData", ARGS_NONE, RET_OBJECT},

    /* WebSockets */
    {"WebSocket", "WebSocket", ARGS_WEBSOCKET, RET_OBJECT},

    /* Encoding */
    {"TextEncoder", "TextEncoder", ARGS_NONE, RET_OBJECT},
    {"TextDecoder", "TextDecoder", ARGS_ENCODING, RET_OBJECT},

    /* Crypto */
    {"crypto.getRandomValues", "crypto.getRandomValues", ARGS_CRYPTO_RANDOM, RET_OBJECT},
    {"crypto.subtle.decrypt", "crypto.subtle.decrypt", ARGS_CRYPTO_SUBTLE, RET_PROMISE},
    {"crypto.subtle.deriveBits", "crypto.subtle.deriveBits", ARGS_CRYPTO_SUBTLE, RET_PROMISE},
    {"crypto.subtle.deriveKey", "crypto.subtle.deriveKey", ARGS_CRYPTO_SUBTLE, RET_PROMISE},
    {"crypto.subtle.digest", "crypto.subtle.digest", ARGS_CRYPTO_SUBTLE, RET_PROMISE},
    {"crypto.subtle.encrypt", "crypto.subtle.encrypt", ARGS_CRYPTO_SUBTLE, RET_PROMISE},
    {"crypto.subtle.exportKey", "crypto.subtle.exportKey", ARGS_CRYPTO_SUBTLE, RET_PROMISE},
    {"crypto.subtle.generateKey", "crypto.subtle.generateKey", ARGS_CRYPTO_SUBTLE, RET_PROMISE},
    {"crypto.subtle.importKey", "crypto.subtle.importKey", ARGS_CRYPTO_SUBTLE, RET_PROMISE},
    {"crypto.subtle.sign", "crypto.subtle.sign", ARGS_CRYPTO_SUBTLE, RET_PROMISE},
    {"crypto.subtle.unwrapKey", "crypto.subtle.unwrapKey", ARGS_CRYPTO_SUBTLE, RET_PROMISE},
    {"crypto.subtle.verify", "crypto.subtle.verify", ARGS_CRYPTO_SUBTLE, RET_PROMISE},
    {"crypto.subtle.wrapKey", "crypto.subtle.wrapKey", ARGS_CRYPTO_SUBTLE, RET_PROMISE},

    /* Performance */
    {"performance.now", "performance.now", ARGS_NONE, RET_NUMBER},
    {"performance.mark", "performance.mark", ARGS_STRING, RET_NONE},
    {"performance.measure", "performance.measure", ARGS_OBJECT, RET_NONE},

    /* Events */
    {"EventTarget", "EventTarget", ARGS_NONE, RET_OBJECT},
    {"Event", "Event", ARGS_EVENT, RET_OBJECT},
    {"ErrorEvent", "ErrorEvent", ARGS_EVENT, RET_OBJECT},
    {"CloseEvent", "CloseEvent", ARGS_EVENT, RET_OBJECT},
    {"MessageEvent", "MessageEvent", ARGS_EVENT, RET_OBJECT},

    /* Storage */
    {"indexedDB", "indexedDB", ARGS_STORAGE, RET_OBJECT},
    {"caches", "caches", ARGS_STORAGE, RET_OBJECT},

    {NULL, NULL, 0, 0} /* Sentinel */
};

#endif

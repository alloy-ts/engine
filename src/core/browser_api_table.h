#ifndef ALLOY_BROWSER_API_TABLE_H
#define ALLOY_BROWSER_API_TABLE_H

#include <stddef.h>

typedef enum {
    ARGS_NONE,
    ARGS_FETCH,
    ARGS_URL,
    ARGS_WS,
    ARGS_DIGEST,
    ARGS_ENCRYPT,
    ARGS_DECRYPT,
    ARGS_GENKEY,
    ARGS_SIGN,
    ARGS_VERIFY,
    ARGS_IMPORTKEY,
    ARGS_EXPORTKEY,
    ARGS_RANDBUF,
} AlloyArgSchema;

typedef enum {
    RET_VOID,
    RET_OBJECT,
    RET_RESPONSE,
    RET_ARRAYBUF,
    RET_CRYPTOKEY,
    RET_BOOL,
    RET_F64,
} AlloyReturnSchema;

typedef struct {
    const char *js_name;
    const char *webview_method;
    AlloyArgSchema arg_schema;
    AlloyReturnSchema return_schema;
} AlloyBrowserAPIEntry;

static const AlloyBrowserAPIEntry ALLOY_BROWSER_API_TABLE[] = {
    /* HTTP */
    { "fetch",                       "fetch",                       ARGS_FETCH,    RET_RESPONSE  },
    { "Request",                     "Request",                     ARGS_NONE,     RET_OBJECT    },
    { "Response",                    "Response",                    ARGS_NONE,     RET_OBJECT    },
    { "Headers",                     "Headers",                     ARGS_NONE,     RET_OBJECT    },
    { "AbortController",             "AbortController",             ARGS_NONE,     RET_OBJECT    },
    { "AbortSignal",                 "AbortSignal",                 ARGS_NONE,     RET_OBJECT    },
    /* URLs */
    { "URL",                         "URL",                         ARGS_URL,      RET_OBJECT    },
    { "URLSearchParams",             "URLSearchParams",             ARGS_NONE,     RET_OBJECT    },
    /* Web Workers */
    { "Worker",                      "Worker",                      ARGS_NONE,     RET_OBJECT    },
    { "MessagePort",                 "MessagePort",                 ARGS_NONE,     RET_OBJECT    },
    { "MessageChannel",              "MessageChannel",              ARGS_NONE,     RET_OBJECT    },
    { "BroadcastChannel",            "BroadcastChannel",            ARGS_NONE,     RET_OBJECT    },
    /* Streams */
    { "ReadableStream",              "ReadableStream",              ARGS_NONE,     RET_OBJECT    },
    { "WritableStream",              "WritableStream",              ARGS_NONE,     RET_OBJECT    },
    { "TransformStream",             "TransformStream",             ARGS_NONE,     RET_OBJECT    },
    { "ByteLengthQueuingStrategy",   "ByteLengthQueuingStrategy",   ARGS_NONE,     RET_OBJECT    },
    { "CountQueuingStrategy",        "CountQueuingStrategy",        ARGS_NONE,     RET_OBJECT    },
    /* Blob */
    { "Blob",                        "Blob",                        ARGS_NONE,     RET_OBJECT    },
    { "File",                        "File",                        ARGS_NONE,     RET_OBJECT    },
    { "FormData",                    "FormData",                    ARGS_NONE,     RET_OBJECT    },
    /* WebSockets */
    { "WebSocket",                   "WebSocket",                   ARGS_WS,       RET_OBJECT    },
    /* Encoding */
    { "TextEncoder",                 "TextEncoder",                 ARGS_NONE,     RET_OBJECT    },
    { "TextEncoderStream",           "TextEncoderStream",           ARGS_NONE,     RET_OBJECT    },
    { "TextDecoder",                 "TextDecoder",                 ARGS_NONE,     RET_OBJECT    },
    { "TextDecoderStream",           "TextDecoderStream",           ARGS_NONE,     RET_OBJECT    },
    /* Crypto */
    { "crypto.subtle.digest",        "crypto.subtle.digest",        ARGS_DIGEST,   RET_ARRAYBUF  },
    { "crypto.subtle.encrypt",       "crypto.subtle.encrypt",       ARGS_ENCRYPT,  RET_ARRAYBUF  },
    { "crypto.subtle.decrypt",       "crypto.subtle.decrypt",       ARGS_DECRYPT,  RET_ARRAYBUF  },
    { "crypto.subtle.generateKey",   "crypto.subtle.generateKey",   ARGS_GENKEY,   RET_CRYPTOKEY },
    { "crypto.subtle.sign",          "crypto.subtle.sign",          ARGS_SIGN,     RET_ARRAYBUF  },
    { "crypto.subtle.verify",        "crypto.subtle.verify",        ARGS_VERIFY,   RET_BOOL      },
    { "crypto.subtle.importKey",     "crypto.subtle.importKey",     ARGS_IMPORTKEY,RET_CRYPTOKEY },
    { "crypto.subtle.exportKey",     "crypto.subtle.exportKey",     ARGS_EXPORTKEY,RET_ARRAYBUF  },
    { "crypto.getRandomValues",      "crypto.getRandomValues",      ARGS_RANDBUF,  RET_ARRAYBUF  },
    /* Performance */
    { "performance.now",             "performance.now",             ARGS_NONE,     RET_F64       },
    { "performance.mark",            "performance.mark",            ARGS_NONE,     RET_VOID      },
    { "performance.measure",         "performance.measure",         ARGS_NONE,     RET_VOID      },
    /* Events */
    { "EventTarget",                 "EventTarget",                 ARGS_NONE,     RET_OBJECT    },
    { "Event",                       "Event",                       ARGS_NONE,     RET_OBJECT    },
    { "ErrorEvent",                  "ErrorEvent",                  ARGS_NONE,     RET_OBJECT    },
    { "CloseEvent",                  "CloseEvent",                  ARGS_NONE,     RET_OBJECT    },
    { "MessageEvent",                "MessageEvent",                ARGS_NONE,     RET_OBJECT    },
    /* IndexedDB / Cache */
    { "indexedDB",                   "indexedDB",                   ARGS_NONE,     RET_OBJECT    },
    { "caches",                      "caches",                      ARGS_NONE,     RET_OBJECT    },
    { NULL, NULL, 0, 0 }  /* sentinel */
};

#endif

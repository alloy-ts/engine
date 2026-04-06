# AlloyScript Architecture

## Overview

AlloyScript is a minimal, secure, local-first JavaScript runtime for desktop (macOS + Windows). It runs three cooperating processes, each with a distinct role and strict isolation boundary:

```
alloy_host  (trusted core — MicroQuickJS + user JS logic)
    │
    ├──► alloy_web   (hidden WebView sidecar — browser Web APIs)
    │    X25519 + XChaCha20-Poly1305 IPC
    │    Invisible 1×1 WebView. Never used for rendering.
    │
    └──► alloy_gui   (native UI process — windows, controls, layout)
         X25519 + XChaCha20-Poly1305 IPC
         AppKit on macOS, Win32/WinUI on Windows.
         Spawned lazily on first alloy.ui.* call.
         ↓
         alloy_renderer  (optional isolated window child, per-window)
```

**Hard isolation rules:**
- `alloy_web` and `alloy_gui` never communicate with each other.
- `alloy_host` contains zero platform GUI symbols when `ALLOY_ENABLE_GUI=OFF`.
- `alloy_host` contains zero WebView symbols when `ALLOY_ENABLE_WEB=OFF`.
- ThinLTO strips each sidecar's code independently.

---

## Process Model

### alloy_host — Trusted Core

- Owns the MicroQuickJS context and all user JS execution.
- Holds the central capability store; every IPC request is capability-checked here.
- Spawns `alloy_web` lazily on first Web API call.
- Spawns `alloy_gui` lazily on first `alloy.ui.*` call (when `ALLOY_ENABLE_GUI=ON`).
- Entry point: `src/core/main_host.c`.

### alloy_web — Hidden WebView Sidecar

- Contains an invisible 1×1 system WebView (WKWebView on macOS, WebView2 on Windows).
- Provides real browser Web APIs: `fetch`, `crypto`, `URL`, `TextEncoder`, streams, WebSocket.
- Sandboxed at the OS level (macOS `sandbox_init`, Windows AppContainer).
- Never used for rendering. Never communicates with `alloy_gui`.
- Entry point: `src/web/webview_process.c`.

### alloy_gui — Native UI Process (Phase 2)

- Runs all native UI: windows, controls, layout engine.
- Pure native controls (AppKit on macOS, Win32/WinUI on Windows).
- A visible `<WebView>` control is only created when the user explicitly writes `<WebView>` in JSX.
- Communicates with `alloy_host` over its own encrypted IPC channel.
- Entry point: `src/gui/alloy_gui_main.c`.

### alloy_renderer — Isolated Window Child (optional)

- Spawned by `alloy_gui` when `<Window process={{ isolated: true }}>` is declared.
- Further sandboxed (macOS `sandbox_init`, Windows AppContainer).
- Entry point: `src/gui/alloy_renderer_main.c`.

---

## AlloyJSEngine Vtable

Defined in `include/alloy/js_engine.h`. Abstracts MicroQuickJS and WebView behind five function pointers with a strict **JSON string boundary** — no engine-specific type (`JSValue`, `JSContext*`, `webview_t`) ever crosses this interface.

```c
typedef struct AlloyJSEngine {
    void *engine;
    char *(*call_fn)   (AlloyJSEngine*, const char *fn,   const char *args_json);
    char *(*eval_fn)   (AlloyJSEngine*, const char *src,  const char *filename);
    int   (*bind_fn)   (AlloyJSEngine*, const char *name, alloy_handler_fn, void*);
    int   (*emit_fn)   (AlloyJSEngine*, const char *event,const char *payload_json);
    void  (*destroy_fn)(AlloyJSEngine*);
    bool  destroyed;
} AlloyJSEngine;
```

| Direction | MQJSAdapter | WebViewAdapter |
|---|---|---|
| C→JS eval | `JS_Eval` with `JS_EVAL_RETVAL` | `webview_eval` wrapped in IIFE |
| C→JS call | eval-based dotted name resolution | `fn.apply(null, args)` via eval |
| JS→C bind | `JS_NewCFunctionParams` at `USER+0` | `webview_bind` |
| C→JS emit | eval `__alloy_emit(event, payload)` | `webview_eval("__alloy_emit(...)")` |

Adapters live in `src/core/backend_adapters/`.

---

## Encrypted IPC

Defined in `include/alloy/ipc_proto.h`, implemented in `src/core/ipc_channel.c`.

- **Transport**: Unix domain socket (macOS/Linux) or named pipe (Windows).
- **Handshake**: X25519 key exchange via libsodium `crypto_kx`.
- **Per-message**: XChaCha20-Poly1305 via libsodium `crypto_secretstream`, with monotonic sequence numbers for replay detection.
- **Wire format**: 4-byte length prefix + encrypted frame. Plaintext layout: `[1B type][8B seq][32B cap_id][payload]`.

Each sidecar has its own independent IPC channel. `alloy_web` and `alloy_gui` channels are completely separate.

---

## Capability Model

Defined in `include/alloy/capability.h`, implemented in `src/core/capability.c`.

Every IPC request from any sidecar must carry a `cap_id`. The core validates it before acting:

- Capabilities are issued by the core with a resource name, permission bitmask, scope patterns, and optional expiry.
- `web:fetch` — network access for `alloy_web`.
- `ui:window`, `ui:control`, `ui:navigate` — window and control operations for `alloy_gui`.
- Scope matching supports exact strings and wildcard prefixes.
- Cap ID comparison is constant-time to prevent timing attacks.
- Revocation is immediate and in-memory.

---

## Plugin System

Defined in `include/alloy/plugin.h`. Every subsystem is a `RuntimePlugin` with up to five nullable hooks:

```c
struct RuntimePlugin {
    const char *name;
    int  (*on_init)      (RuntimePlugin*, AlloyJSEngine*);
    void (*on_eval_begin)(RuntimePlugin*, AlloyJSEngine*, const char *source);
    void (*on_eval_end)  (RuntimePlugin*, AlloyJSEngine*, const char *result_json);
    void (*on_destroy)   (RuntimePlugin*, AlloyJSEngine*);
    int  (*on_parse)     (RuntimePlugin*, const char *source, size_t len,
                          const char *filename, char **out);  /* build-time */
    void *userdata;
};
```

### Hook semantics

| Hook | When | Used by |
|---|---|---|
| `on_init` | Once, during `alloy_engine_create_with_config()` | All plugins — register bridges, spawn processes |
| `on_eval_begin` | Before every `alloy_runtime_eval()` | `alloy:config` — snapshots config state |
| `on_eval_end` | After every `alloy_runtime_eval()` | `alloy:config` — post-eval cleanup |
| `on_destroy` | During `alloy_engine_destroy_runtime()`, reverse order | All plugins — shutdown processes, revoke caps |
| `on_parse` | **Build-time** — Bun macro pipeline, before ES5 pass | `alloy:gui` — JSX → `alloy.ui.*` calls |

### Hook assignment by plugin

```
Plugin         on_init   on_eval_begin  on_eval_end  on_destroy  on_parse
─────────────  ────────  ─────────────  ───────────  ──────────  ────────
alloy:config   spawn     snapshot       cleanup      teardown    —
alloy:web      spawn     —              —            shutdown    —
alloy:gui      spawn*    —              —            shutdown    JSX→ui
               + IPC reader thread
```

`*` alloy:gui `on_init` spawns `alloy_gui`, performs IPC handshake, binds `alloy.ui.*` bridges, and starts the IPC reader thread that dispatches window events to JS callbacks.

### Engine init pipeline

```
alloy_engine_create_with_config()
  1. mmap 16 MiB JS heap
  2. JS_NewContext (URN + DID stdlib)
  3. Wrap in AlloyJSEngine via MQJSAdapter
  4. Eval runtime_bootstrap.ts (Alloy global, event system, plugin registry)
  5. Create capability store
  6. load_plugin(alloy:config)  → on_init: Alloy.config in JS, __alloy_use_config bridge
  7. load_plugin(alloy:web)     → on_init: spawn alloy_web lazily, __alloy_web_invoke bridge
  8. load_plugin(alloy:gui)     → on_init: spawn alloy_gui, bind alloy.ui.*, start IPC reader
     [only when ALLOY_ENABLE_GUI=ON and noGuiAPI=false]
```

### Window event callback flow

Window events (`onInit`, `onResize`, `onClose`, etc.) are **user-defined JS callbacks** on a specific window instance — not `RuntimePlugin` hooks.

```
User writes JSX:
  <Window onInit={handleInit} onResize={handleResize}>

Bun macro (on_parse) compiles to:
  var _w = alloy.ui.createWindow({
    onInit: handleInit,    // stored in WindowCallbackTable[windowId]
    onResize: handleResize,
  }, cap);

At runtime:
  alloy_gui detects OS resize event
    → sends ALLOY_MSG_GUI_EVENT {type:"resize", windowId:1, width:900, height:700}
  IPC reader thread in gui_plugin.c receives it
    → looks up WindowCallbackTable[1]["onResize"]
    → calls alloy_engine_call(engine, fn_ref, event_json)
    → handleResize({width:900, height:700}) fires in MicroQuickJS
```

No dedicated event process is needed. The IPC reader thread in `alloy_host` handles all dispatch. `alloy_gui` only sends events; it never executes JS.

## Transpiler Pipeline

User TypeScript/JSX goes through four passes before execution in MicroQuickJS:

```
User .ts/.tsx
    │
    ▼ Pass 0 (build-time, Bun macro / on_parse)
    │  alloy:gui JSX → alloy.ui.* calls
    │  <Window onResize={fn}> → alloy.ui.createWindow({onResize:fn}, cap)
    │  Compile-time error if <WebView> has no <Window> ancestor
    │
    ▼ Pass 1 (Bun.Transpiler, target="browser")
    │  TypeScript → modern JS (strips types, handles imports)
    │  async/await syntax is preserved here — Bun does NOT downlevel it
    │  because MicroQuickJS has no native Promise; we handle it in Pass 3
    │
    ▼ Pass 2 (ES5 downlevel — text-level regex transforms)
    │  const/let → var
    │  Arrow functions → function expressions
    │  Template literals → string concatenation
    │  for..of → indexed for loop
    │  Spread/rest → apply/slice
    │  Shorthand methods → function properties
    │  Default parameters → manual undefined checks
    │
    ▼ Pass 3 (async polyfill — generator-to-Promise pattern)
       async function f(p) { ... }
         → function f(p) { return __alloy_spawn(function*() { ... }); }
       await expr
         → yield expr
       Injects __alloy_spawn and process globals into the output
```

### Why generators, not native async/await

MicroQuickJS is strictly ES5-ish. `async` and `await` are **syntax-level keywords**
that the parser rejects. They cannot be polyfilled at runtime — they must be
transformed at the source level before the code reaches MicroQuickJS.

The generator-to-Promise pattern is the correct approach:

1. **Transpile**: `async function` → `function*` (generator), `await` → `yield`
2. **Inject `__alloy_spawn`**: a runtime helper that drives the generator, calling
   `.next()` each time a yielded Promise resolves
3. **Inject `process`**: the Node/Bun-compatible global, backed by C

### `__alloy_spawn` — the coroutine driver

Injected into every MicroQuickJS context during bootstrap:

```js
function __alloy_spawn(genF) {
  return new Promise(function(resolve, reject) {
    var gen = genF();
    function step(nextF) {
      var next;
      try { next = nextF(); } catch(e) { return reject(e); }
      if (next.done) return resolve(next.value);
      Promise.resolve(next.value).then(
        function(v) { step(function() { return gen.next(v); }); },
        function(e) { step(function() { return gen.throw(e); }); }
      );
    }
    step(function() { return gen.next(); });
  });
}
```

For IPC-backed operations (fetch, crypto, etc.), the yielded value is a task
descriptor `{ __alloy_task: "ipc", payload: "...", task_id: 42 }`. The C event
loop (`AlloyEventLoop`) parks the coroutine, dispatches the IPC call to the
background I/O thread, and resumes the generator when the response arrives.

### `process` global

Injected by `engine_create.c` during MicroQuickJS initialisation:

```c
JSValue process = JS_NewObject(ctx);

// Static platform data
JS_SetPropertyStr(ctx, process, "platform",
    JS_NewString(ctx, ALLOY_PLATFORM_STRING));  /* "darwin" | "win32" */
JS_SetPropertyStr(ctx, process, "version",
    JS_NewString(ctx, ALLOY_VERSION));

// argv — populated from C main() arguments
JS_SetPropertyStr(ctx, process, "argv",
    alloy_build_argv_array(ctx, argc, argv));

// env — reads from the encrypted environment store
JS_SetPropertyStr(ctx, process, "env",
    alloy_get_encrypted_env(ctx));

// nextTick — bridges to the MicroQuickJS job queue
JS_SetPropertyStr(ctx, process, "nextTick",
    JS_NewCFunction(ctx, alloy_next_tick, "nextTick", 1));

// stdout / stderr — redirect to alloy_host log output
JS_SetPropertyStr(ctx, process, "stdout",
    alloy_build_stream_obj(ctx, ALLOY_STREAM_STDOUT));
JS_SetPropertyStr(ctx, process, "stderr",
    alloy_build_stream_obj(ctx, ALLOY_STREAM_STDERR));

// memoryUsage() — reports MicroQuickJS fixed-buffer utilisation
JS_SetPropertyStr(ctx, process, "memoryUsage",
    JS_NewCFunction(ctx, alloy_memory_usage, "memoryUsage", 0));

// process.alloy — AlloyScript-specific extensions
JS_SetPropertyStr(ctx, process, "alloy",
    alloy_build_alloy_namespace(ctx));

JS_SetPropertyStr(ctx, global_obj, "process", process);
```

`process.nextTick(fn)` pushes `fn` into a C++ queue and executes it after the
current script finishes via `JS_RunJob`. This is critical for Promise resolution
ordering and matches Node/Bun semantics.

`process.memoryUsage()` returns `{ heapUsed, heapTotal }` where `heapTotal` is
the fixed MicroQuickJS buffer size and `heapUsed` is the current allocation.

---

## Async Model

MicroQuickJS has no native event loop and rejects `async`/`await` syntax. AlloyScript
implements async/await via the **generator-to-Promise pattern** driven by a background
I/O thread.

### The pattern

```
User writes:                    Transpiler produces:
─────────────────────────────   ──────────────────────────────────────────────
async function loadData() {     function loadData() {
  const r = await fetch(url);     return __alloy_spawn(function*() {
  return r.json();                  var r = yield fetch(url);
}                                   return r.json();
                                  });
                                }
```

`__alloy_spawn` drives the generator: each time a yielded value resolves, it calls
`gen.next(resolvedValue)` to resume the coroutine.

### IPC-backed operations

For operations that cross process boundaries (fetch, crypto, etc.), the yielded value
is a task descriptor. The C event loop parks the coroutine and resumes it when the
I/O thread posts the completion:

```
JS thread (main)                    I/O thread (background)
────────────────────────────        ──────────────────────────────────
var r = yield fetch(url)            waits on task queue (condvar)
  ↓ yields task descriptor:
  { __alloy_task: "ipc",            ← wakes, dequeues task
    payload: '{"method":"fetch"…}', → alloy_ipc_send() to alloy_web
    task_id: 42 }                   → alloy_ipc_recv() [blocking here]
  ↓                                 → posts AlloyCompletion{task_id:42}
C parks coroutine 42                → signals comp_cv
C runs other ready coroutines
C drains completion queue
C resumes coroutine 42 with result
  ↓
gen.next(response) → r = response
```

`Promise.all` dispatches all tasks before waiting — total time ≈ `max(t_a, t_b)`.

### Components

- `include/alloy/event_loop.h` / `src/core/event_loop.c` — background I/O thread, task/completion queues
- `src/core/transpiler.ts` — Pass 3: rewrites `async`/`await` → generator + `__alloy_spawn`
- `src/core/runtime_bootstrap.ts` — injects `__alloy_spawn` and `process` into MicroQuickJS global scope
- `alloy_next_tick` C function — bridges `process.nextTick` to the MicroQuickJS job queue

---

## URN and DID Classes

Implemented in `src/web/urn.c` and `src/web/did.c`, registered in `src/core/stdlib.c`.

```
JS_CLASS_USER + 0  →  URN   urn:<nid>:<nss>          (RFC 8141)
JS_CLASS_USER + 1  →  DID   did:<method>:<method-id>  (W3C DID Core 1.0)
```

---

## MicroQuickJS Notes

MicroQuickJS (`packages/mquickjs/`) is a minimal fork of QuickJS with a compacting GC and a fixed-size memory buffer. Key differences:

- No `JS_FreeValue` / `JS_DupValue` — GC manages all lifetimes.
- No `JS_IsArray` / `JS_IsObject` as public API — use `JS_GetClassID`.
- No `JS_GetContextOpaque` — provided by `src/core/mqjs_compat.c`.
- JSON operations go through `JS_Eval` with `JS_EVAL_JSON` flag.
- Stdlib table generated at build time by `mqjs_stdlib_builder`.

Compatibility shims: `include/alloy/mqjs_compat.h`.

---

## Build

```bash
# Phase 1: core + web APIs
cmake -B build -DCMAKE_BUILD_TYPE=Release \
  -DALLOY_ENABLE_WEB=ON \
  -DALLOY_ENABLE_GUI=OFF
cmake --build build -j$(nproc)

# Phase 1 + Phase 2: core + web APIs + native GUI
cmake -B build-gui -DCMAKE_BUILD_TYPE=Release \
  -DALLOY_ENABLE_WEB=ON \
  -DALLOY_ENABLE_GUI=ON
cmake --build build-gui -j$(nproc)

# Headless (no web, no GUI — smallest binary)
cmake -B build-headless -DCMAKE_BUILD_TYPE=Release \
  -DALLOY_ENABLE_WEB=OFF \
  -DALLOY_ENABLE_GUI=OFF
cmake --build build-headless -j$(nproc)
```

ThinLTO is enabled automatically. libsodium is always required. WebKit/WebView2 is required when `ALLOY_ENABLE_WEB=ON` or `ALLOY_ENABLE_GUI=ON`.

---

## Source Layout

```
include/alloy/
  js_engine.h          # AlloyJSEngine vtable + inline NULL-guards
  plugin.h             # RuntimePlugin (on_init/on_eval_begin/on_eval_end/on_destroy/on_parse)
  capability.h         # Capability token + store API
  ipc_proto.h          # Wire protocol, AlloyIPCChannel API
  config.h             # AlloyRuntimeConfig (no_web_api, no_gui_api, log_level, heap_size)
  gui.h                # AlloyGuiBackend vtable, AlloyWindowOptions, AlloyControlSpec
  mqjs_compat.h        # JS_GetContextOpaque, JS_FreeCString shims
  runtime.h            # AlloyRuntime public API (alloy_runtime_eval, plugin accessors)
  event_loop.h         # AlloyEventLoop (background I/O thread + task/completion queues)

src/
  core/
    main_host.c        # alloy_host entry point — alloy_engine_create + alloy_runtime_eval
    engine_create.c    # AlloyRuntime init + plugin pipeline + hook dispatch
    capability.c       # Capability store (issue, validate, revoke, constant-time)
    ipc_channel.c      # Encrypted IPC (X25519 handshake + XChaCha20-Poly1305)
    event_loop.c       # Background I/O thread + task/completion queues
    embedded_js.c      # Embeds bootstrap + shims as C string literals
    mqjs_compat.c      # JS_GetContextOpaque via mquickjs_priv.h
    stdlib.c           # MQJS stdlib table (URN/DID + full stdlib)
    transpiler.ts      # TS→ES5 pipeline + async/await→generator rewrite
    runtime_bootstrap.ts  # Alloy global, event system, plugin registry (TS source)
    backend_adapters/
      mqjs_adapter.c   # MicroQuickJS → AlloyJSEngine
      webview_adapter.c# WebView → AlloyJSEngine
    plugins/
      config/
        config_plugin.c# alloy:config (Alloy.config, __alloy_use_config, __alloy_config_set)

  web/
    webview_process.c  # alloy_web entry point (hidden WebView sidecar)
    web_runtime.c      # alloy:web plugin (on_init: spawn sidecar, bind __alloy_web_invoke)
    urn.c / urn.h      # URN class (RFC 8141)
    did.c              # DID class (W3C DID Core 1.0)

  gui/                 # Phase 2 — compiled only when ALLOY_ENABLE_GUI=ON
    alloy_gui_main.c   # alloy_gui entry point (native UI process)
    alloy_renderer_main.c  # alloy_renderer entry point (isolated window child)
    gui_plugin.c       # alloy:gui plugin (on_init: spawn alloy_gui + IPC reader thread)
    gui_shims.ts       # alloy.ui.* JS API (TS source, transpiled to ES5)
    layout_engine.cpp  # Flex-like layout engine (positions native controls)
    backends/
      cocoa_backend.m  # macOS: AppKit (NSWindow, NSButton, WKWebView, NSMenu...)
      win32_backend.c  # Windows: Win32/WinUI (CreateWindowEx, WebView2, Shell_NotifyIcon...)
      mock_backend.c   # Headless mock for CI tests

  shims/
    web_api_shims.ts   # Web API proxies (fetch, URL, crypto, streams...) — TS source
    ui/
      macros.ts        # Bun macro: JSX → alloy.ui.* calls (on_parse equivalent)
      types.ts         # TypeScript interfaces for JSX element props
      jsx.d.ts         # JSX namespace declaration

tests/
  test_core.c              # Vtable, capability, IPC unit tests
  test_plugin_lifecycle.c  # Plugin hook contract tests
  test_gui_plugin.c        # alloy:gui unit tests (mock backend)
  test_gui_properties.c    # alloy:gui property-based tests

packages/
  mquickjs/            # MicroQuickJS submodule
  webview/             # webview submodule (WKWebView / WebView2 / WebKitGTK)
```

# AlloyScript

A minimal, secure, local-first JavaScript runtime for desktop (macOS + Windows).
Runs modern TypeScript natively, compiles to self-contained native binaries, and
provides real browser Web APIs and native UI — all without Electron or Node.js.

---

## What it is

AlloyScript runs three cooperating OS processes, each with a distinct role:

```
alloy_host   Trusted core. Runs your TypeScript/JS in MicroQuickJS.
    │         Owns the capability store. Orchestrates everything.
    │
    ├──►  alloy_web   Hidden 1×1 WebView. Provides fetch, crypto, streams,
    │                 WebSocket, URL, TextEncoder, etc. Never renders UI.
    │                 Sandboxed. Communicates only with alloy_host.
    │
    └──►  alloy_gui   Native UI process. AppKit on macOS, Win32/WinUI on Windows.
                      Pure native controls. Spawned on first alloy.ui.* call.
                      Window events (onResize, onClose, etc.) are dispatched
                      back to your JS callbacks via an IPC reader thread.
```

**Hard rules:**
- `alloy_web` and `alloy_gui` never communicate with each other.
- `alloy_host` contains zero platform GUI symbols when `ALLOY_ENABLE_GUI=OFF`.
- `alloy_host` contains zero WebView symbols when `ALLOY_ENABLE_WEB=OFF`.
- ThinLTO strips each sidecar's code independently.

---

## Quick start

```bash
# Prerequisites
brew install libsodium cmake bun   # macOS
# apt install libsodium-dev cmake pkg-config libgtk-3-dev libwebkit2gtk-4.0-dev  # Linux

# Build (Phase 1: core + web APIs)
cmake -B build -DCMAKE_BUILD_TYPE=Release -DALLOY_ENABLE_WEB=ON
cmake --build build -j$(nproc)

# Run a script
./build/alloy_host my_script.js

# Or use the CLI
bun run src/main.ts run my_script.ts
```

---

## CLI

```
alloy <command> [options]

Commands:
  run      <script.ts>              Run a TypeScript file
  compile  <input.ts> [-o <dir>]    Compile to a self-contained native binary
  transpile <input.ts> [--out <f>]  Transpile TS to ES5 for MicroQuickJS
  repl                              Interactive REPL
  init     [name]                   Scaffold a new project
  test                              Run tests

Options:
  --no-web          Disable alloy:web sidecar (smaller binary)
  --build-type      release | debug  (default: release)
  -o, --outdir      Output directory
  -h, --help        Show help
```

---

## Writing AlloyScript

AlloyScript accepts modern TypeScript. The transpiler handles the ES5 downlevel
and async/await polyfill automatically.

```typescript
// fetch works via the hidden alloy:web sidecar
const resp = await fetch("https://api.example.com/data");
const data = await resp.json();
console.log(data);

// URN and DID are first-class types
const id = new DID("did:example:123abc");
console.log(id.method);    // "example"
console.log(id.methodId);  // "123abc"
```

### Native UI (Phase 2, `ALLOY_ENABLE_GUI=ON`)

JSX is transformed at build time by the Bun macro — no runtime JSX overhead.

```tsx
import { Window, Stack, Button, Label } from "alloy/ui" with { type: "macro" };

const cap = await alloy.ui.requestCapability("ui:window", ["create", "control"]);

<Window
  title="My App"
  width={800}
  height={600}
  onInit={() => console.log("window ready")}
  onResize={({ width, height }) => console.log("resized", width, height)}
  onClose={() => process.exit(0)}
  capId={cap}
>
  <Stack direction="vertical" gap={8} padding={16}>
    <Label text="Hello from AlloyScript" />
    <Button label="Click me" onClick={() => console.log("clicked")} />
  </Stack>
</Window>
```

Window event callbacks (`onInit`, `onResize`, `onClose`, etc.) are stored in
`alloy_host` and invoked in MicroQuickJS when `alloy_gui` sends the corresponding
OS event over IPC. No dedicated event process is needed.

---

## Configuration

`alloyconfig.toml` at the project root:

```toml
target    = "local-first"   # "local-first" | "server-first"
logLevel  = "warn"          # "debug" | "warn" | "error"
noWebAPI  = false           # true = disable alloy:web sidecar
noGuiAPI  = false           # true = disable alloy:gui process

[test]
coverage          = true
coverageThreshold = { line = 80, function = 75 }

[plugins.my-plugin]
dbPath = "./data/app.db"
```

### Plugin config API

```typescript
// In your plugin's on_init JS layer:
const config = useConfig("my-plugin", {
  dbPath:         { default: "./data/app.db" },
  maxConnections: { default: 10 },
});

console.log(config.dbPath);
config.set("maxConnections", 20);
config.set("dbPath", "./prod.db", { persist: true });
```

---

## Plugin system

Every AlloyScript capability is a `RuntimePlugin` with up to five nullable hooks:

| Hook | When | Example use |
|---|---|---|
| `on_init` | Once at engine startup | Spawn sidecar process, bind JS bridges |
| `on_eval_begin` | Before each script eval | Snapshot config state |
| `on_eval_end` | After each script eval | Post-eval cleanup |
| `on_destroy` | Engine teardown (reverse order) | Shutdown process, revoke caps |
| `on_parse` | **Build-time** (Bun macro) | JSX → `alloy.ui.*` calls |

Core plugins load in dependency order:

```
alloy:config  →  alloy:web  →  alloy:gui  →  [third-party plugins]
```

Third-party plugins are declared in `alloyconfig.toml`, resolved from npm or local
paths, and topologically sorted by declared dependencies (Kahn's algorithm).

---

## Security model

- **Encrypted IPC** — X25519 handshake + XChaCha20-Poly1305 per-message encryption
  (libsodium). Each sidecar has its own independent channel.
- **Capability enforcement** — every IPC request carries a `cap_id` validated by
  `alloy_host` before any action is taken. Constant-time comparison.
- **OS sandboxing** — `alloy_web` and `alloy_gui` are sandboxed at the OS level
  (macOS `sandbox_init`, Windows AppContainer).
- **Replay protection** — monotonic sequence numbers on every IPC message.
- **ThinLTO dead-code elimination** — unused sidecars are stripped from the binary.

---

## Build options

```bash
# Phase 1: core + web APIs (default)
cmake -B build -DALLOY_ENABLE_WEB=ON -DALLOY_ENABLE_GUI=OFF

# Phase 1 + Phase 2: core + web APIs + native GUI
cmake -B build-gui -DALLOY_ENABLE_WEB=ON -DALLOY_ENABLE_GUI=ON

# Headless: no web, no GUI (smallest binary)
cmake -B build-headless -DALLOY_ENABLE_WEB=OFF -DALLOY_ENABLE_GUI=OFF

cmake --build <dir> -j$(nproc)
```

---

## Source layout

```
src/
  core/
    main_host.c          Entry point for alloy_host
    engine_create.c      Runtime init + plugin pipeline
    transpiler.ts        TS→ES5 + async/await polyfill
    runtime_bootstrap.ts Alloy global, event system, plugin registry
    plugins/config/      alloy:config plugin
    backend_adapters/    MQJSAdapter, WebViewAdapter

  web/
    webview_process.c    Entry point for alloy_web (hidden WebView sidecar)
    web_runtime.c        alloy:web plugin
    urn.c / did.c        URN (RFC 8141) and DID (W3C) classes

  gui/                   Phase 2 — ALLOY_ENABLE_GUI=ON only
    alloy_gui_main.c     Entry point for alloy_gui (native UI process)
    gui_plugin.c         alloy:gui plugin (IPC reader thread, callback table)
    gui_shims.ts         alloy.ui.* JS API
    layout_engine.cpp    Flex-like layout engine
    backends/            cocoa_backend.m, win32_backend.c, mock_backend.c

  shims/
    web_api_shims.ts     fetch, URL, crypto, streams, WebSocket shims
    ui/macros.ts         Bun macro: JSX → alloy.ui.* (build-time)

include/alloy/
  js_engine.h            AlloyJSEngine vtable
  plugin.h               RuntimePlugin interface (5 hooks)
  capability.h           Capability store API
  ipc_proto.h            IPC wire protocol
  config.h               AlloyRuntimeConfig
  gui.h                  AlloyGuiBackend vtable (Phase 2)
  runtime.h              AlloyRuntime public API
  event_loop.h           Background I/O thread

tests/
  test_core.c            Vtable, capability, IPC tests
  test_plugin_lifecycle.c Plugin hook contract tests
  test_gui_plugin.c      alloy:gui tests (Phase 2)
```

---

## License

AlloyScript source: CC0 (Public Domain).
MicroQuickJS and webview submodules: MIT.

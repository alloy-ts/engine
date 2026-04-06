# AlloyScript

AlloyScript is a high-security TypeScript/JSX runtime built around a **dual-engine** architecture. It pairs a minimal, sandboxed JS core (MicroQuickJS) with a WebView sidecar for modern browser APIs, connected via an encrypted IPC bridge. It is designed as a drop-in replacement for both Node.js and Bun, with a plugin system that extends the runtime with native capabilities.

## Architecture

AlloyScript uses two cooperating engines:

- **MicroQuickJS (Secure Core)** — executes all user logic in a strict ES5 environment with heap protection, stack canaries, and position-independent execution.
- **WebView (Sidecar)** — provides modern browser APIs (`fetch`, `crypto`, `storage`) via an AES-256-GCM encrypted IPC bridge. Treated as an untrusted process with no filesystem access.

Both engines are abstracted behind the `AlloyJSEngine` vtable — five function pointers with a JSON string boundary. No engine-specific type ever crosses this boundary. Adding a new engine target means implementing exactly five functions in a new adapter file.

### Transpiler Pipeline

User TypeScript/JSX goes through three mandatory passes before execution:

1. TypeScript → ES5 downlevel
2. Async/await → generator-based coroutines (MicroQuickJS has no native event loop)
3. MQJS compatibility pass

### Plugin System

Every AlloyScript capability is delivered through the `RuntimePlugin` interface — four nullable lifecycle hooks (`on_init`, `on_eval_begin`, `on_eval_end`, `on_destroy`). Core plugins load in a fixed order:

```
alloy:ffi → alloy:config → alloy:shell
```

Third-party plugins are declared in `alloyconfig.toml`, resolved from npm packages or local paths, topologically sorted by declared dependencies (Kahn's algorithm, O(N+E)), and loaded in dependency order.

### Security Model

- **E2EE IPC** — AES-256-GCM with a per-session key between engines
- **Heap protection** — `mprotect(PROT_NONE)` between execution windows
- **WASI/WIT sandboxing** — shell capability grants enforced via WasmEdge (default, `local-first` mode)
- **OCI container isolation** — each `shellCapabilities` entry runs in a dedicated ephemeral container (`cgr.dev/chainguard/<name>`)
- **Capability enforcement in C** — security-sensitive logic lives in the C layer of each plugin; the JS layer calls through registered bridge globals only

## Configuration

AlloyScript is configured via `alloyconfig.toml` at the project root (WorkspaceConfig) and optionally `~/.alloyconfig.toml` (UserConfig). WorkspaceConfig takes precedence; CLI flags override both.

```toml
# alloyconfig.toml

target = "local-first"          # "local-first" (Desktop/Mobile applications) | "server-first" (Cloud-native applications)
logLevel = "warn"               # "debug" | "warn" | "error"
shellCapabilities = ["use-git"] # permitted external binaries

[test]
coverage = true
coverageThreshold = { line = 80, function = 75 }

[install]
registry = "https://registry.npmjs.org"
frozenLockfile = false

[plugins.my-plugin]
dbPath = "./data/app.db"
```

### Plugin Config API

Plugins declare their own config entries via `useConfig()` during `on_init`:

```typescript
const config = useConfig({
  configs: {
    dbPath: { default: "./data/app.db" },
    maxConnections: { default: 10 },
  }
})

// Read
console.log(config.dbPath)

// Write in-memory
config.set("maxConnections", 20)

// Write and persist to alloyconfig.toml
config.set("dbPath", "./data/prod.db", { persist: true })
```

User-provided values come from the `[plugins.<name>]` section in `alloyconfig.toml`.

## Getting Started

### Installation

```bash
npm install -g @alloyscript/cli
```

### Run a script

```bash
alloy run src/main.ts
```

or 

```bash
alloy x src/main.ts
```

### Build

```bash
alloy build src/main.ts -o my_app
```

### Test

```bash
alloy test
```

### REPL

```bash
alloy repl
```

## Source Layout

```
src/core/
  js_engine.h                    # AlloyJSEngine vtable
  plugin.h                       # RuntimePlugin interface
  alloy_engine_create.c          # engine init + plugin load pipeline
  __alloy_runtime_bootstrap.js   # Alloy global setup, event dispatcher
  backend_adapters/
    mqjs_adapter.c               # MicroQuickJS → AlloyJSEngine
    webview_adapter.c            # WebView → AlloyJSEngine
    shell_bridge.c               # HostProcess ↔ WasmEdge bridge
  config/
    config.ts                    # AlloyConfig interface + defaults
    loader.ts                    # ConfigLoader: discovery → parse → merge → validate
    validator.ts                 # ConfigValidator: type-check + cross-field rules
    pretty_printer.ts            # AlloyConfig → TOML string (round-trip)
    schema.ts                    # Zod schema
  plugins/
    ffi/    ffi_plugin.c  ffi.js
    config/ config_plugin.c  config.js  use_config.ts
    shell/  shell_plugin.c  shell.js
packages/
  mquickjs/                      # MicroQuickJS submodule
  webview/                       # WebView submodule
```

## License

- AlloyScript Source: CC0 (Public Domain)
- MicroQuickJS & WebView: MIT

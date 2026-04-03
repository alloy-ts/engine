# AlloyScript Architecture

## Dual-Engine Orchestration

AlloyScript pairs a trusted **MicroQuickJS** runtime (secure core) with a hidden **WebView** process (sidecar). Both engines are abstracted behind a uniform `AlloyJSEngine` vtable — no engine-specific type ever crosses the abstraction boundary.

- **Trusted Core (MicroQuickJS)**: Executes all user logic in a strict ES5 environment.
- **Untrusted Sidecar (WebView)**: Provides modern Browser APIs (fetch, crypto, storage) via encrypted IPC.
- **AlloyJSEngine vtable** (`src/core/js_engine.h`): Five function pointers with a JSON string boundary — `call_fn`, `eval_fn`, `bind_fn`, `emit_fn`, `destroy_fn`. The IPC layer, browser API dispatcher, and event system call through this vtable exclusively. Adding a new engine target requires implementing exactly five functions in a new adapter file — nothing else.

### C↔JS Communication

Both engines share the same two communication directions, unified by the vtable:

| Direction | MQJS | WebView |
|---|---|---|
| C→JS call (`call_fn`) | `JS_PushArg` × N + `JS_Call` | `webview_eval` → result bridge |
| C→JS eval (`eval_fn`) | `JS_Eval` with `JS_EVAL_RETVAL` | `webview_eval` wrapped in IIFE |
| JS→C register (`bind_fn`) | `JS_NewCFunctionParams` in global object | `webview_bind` with shim |
| JS→C return | `return JSValue` from C function | `webview_return(w, id, status, result)` |
| C→JS event (`emit_fn`) | `JS_Call` on `__alloy_emit` global | `webview_eval("__alloy_emit(...)")` |

All values crossing the vtable are UTF-8 JSON strings. Adapters live in `src/core/backend_adapters/`.

## Transpiler & Runtime Pipeline

User code (TypeScript/JSX) undergoes a mandatory three-pass transformation before execution:

1. **TypeScript to ES5**: Downlevels modern JS/TS to ES5 syntax.
2. **Async/Await Polyfill**: Mandatory rewrite of async functions to generator-based coroutines. MicroQuickJS has no native event loop; asynchronous calls are managed by the C host through synchronous blocking of the generator until IPC results return.
3. **MQJS Compatibility**: Final pass to ensure all syntax (including `var` usage) matches MicroQuickJS's strict parser.

## Plugin System

AlloyScript's capabilities are delivered through a unified `RuntimePlugin` interface (`src/core/plugin.h`). Every subsystem — including core standard library modules — is a plugin.

### RuntimePlugin lifecycle

Each plugin implements up to four nullable hooks:

- `on_init(engine)` — registers C globals via `bind_fn`, called once after engine init
- `on_eval_begin(engine, source)` — called before each top-level script evaluation
- `on_eval_end(engine, result_json)` — called after each top-level script evaluation
- `on_destroy(engine)` — teardown, called in reverse topological order

### Plugin loading pipeline (`alloy_engine_create`)

1. Allocate `AlloyJSEngine`, init adapter (MQJSAdapter or WebViewAdapter)
2. Evaluate `__alloy_runtime_bootstrap.js` — sets up `globalThis.Alloy`, `Alloy.plugins`, `Alloy.on`
3. Load core plugins in fixed order: `alloy:ffi` → `alloy:config` → `alloy:shell`
4. Read `plugins[]` from `alloyconfig.toml`, resolve manifests (`alloy-plugin.toml`)
5. Topological sort (Kahn's algorithm, O(N+E)) with cycle detection
6. For each third-party plugin in topo order: `dlopen` → `alloy_plugin_init()` → `on_init` → `eval_fn(entry.js)` → register under `Alloy.plugins[name]`
7. Dispatch `onStart` to all plugins

### Core plugins

| Plugin | C layer | JS layer |
|---|---|---|
| `alloy:ffi` | `ffi_plugin.c` — registers `__alloy_ffi_*` bridge globals | `ffi.js` — `dlopen`, `FFIType`, `CString`, `JSCallback`, `ptr`, `read`, `cc` |
| `alloy:config` | `config_plugin.c` — serialises `AlloyConfig` to `Alloy.config` | `config.js` — re-exports `Alloy.config` |
| `alloy:shell` | `shell_plugin.c` — registers `__alloy_shell_exec/escape/braces` | `shell.js` — `$` tagged template, `ShellExpression`, `ShellError` |

Each core plugin uses a two-layer design: the C layer owns capability enforcement; the JS layer owns API ergonomics.

### Third-party plugins

Third-party plugins are npm packages or local directories containing an `alloy-plugin.toml` manifest:

```toml
name = "my-plugin"
version = "1.0.0"
entry = "./index.js"
native = "./libmyplugin.dylib"   # optional
dependencies = ["alloy:config"]  # optional
```

Plugins are isolated: JS entry points are evaluated as modules (not scripts), so top-level declarations are module-scoped. Name conflicts on `globalThis` are detected and rejected before registration.

## Security Model

- **E2EE IPC**: All communication between engines is encrypted using AES-256-GCM with a per-session SessionKey generated via CSPRNG.
- **Heap Protection**: The MicroQuickJS heap is gated with `mprotect(PROT_NONE)` between execution windows.
- **Zero-Trust Sidecar**: WebView has no filesystem access and is treated as a potentially compromised process.
- **Capability enforcement in plugins**: Security-sensitive logic lives in the C layer of each plugin. The JS layer has no direct access to native resources — it calls through the registered bridge globals only.
- **WASI/WIT boundaries**: Shell capability grants are enforced either via WasmEdge WASI (default, `local-first` mode) or natively via kernel primitives (mount namespaces, `seccomp` BPF, `execve` envp scoping) when `disableWasmEdge = true`.
- **OCI container isolation**: Each `shellCapabilities` entry runs in a dedicated ephemeral OCI container (`cgr.dev/chainguard/<name>` or `cgr.dev/chainguard/alpine-base`), providing stronger isolation than OS process boundaries.

## Source Layout

```
src/core/
  js_engine.h                        # AlloyJSEngine vtable + inline wrappers
  plugin.h                           # RuntimePlugin interface
  alloy_engine_create.c              # engine init + PluginLoadPipeline
  __alloy_runtime_bootstrap.js       # Alloy global setup, event dispatcher
  backend_adapters/
    mqjs_adapter.c                   # MicroQuickJS → AlloyJSEngine
    webview_adapter.c                # WebView → AlloyJSEngine
    shell_bridge.c                   # HostProcess ↔ WasmEdge bridge
  plugins/
    ffi/    ffi_plugin.c  ffi.js
    config/ config_plugin.c  config.js
    shell/  shell_plugin.c  shell.js
packages/
  mquickjs/                          # MicroQuickJS submodule
  webview/                           # WebView submodule
```

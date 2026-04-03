# AlloyScript Architecture

## Dual-Engine Orchestration
AlloyScript paired a trusted **MicroQuickJS** runtime (secure core) with a hidden **WebView** process (sidecar).
- **Trusted Core (MicroQuickJS)**: Executes all user logic in a strict ES5 environment.
- **Untrusted Sidecar (WebView)**: Provides modern Browser APIs (fetch, crypto, storage) via encrypted IPC.

## Transpiler & Runtime Pipeline
User code (TypeScript/JSX) undergoes a mandatory three-pass transformation before execution:
1. **TypeScript to ES5**: Downlevels modern JS/TS to ES5 syntax.
2. **Async/Await Polyfill**: Mandatory rewrite of async functions to generator-based coroutines. MicroQuickJS has no native event loop; asynchronous calls are managed by the C host through synchronous blocking of the generator until IPC results return.
3. **MQJS Compatibility**: Final pass to ensure all syntax (including `var` usage) matches MicroQuickJS's strict parser.

## Security Model
- **E2EE IPC**: All communication between engines is encrypted using AES-256-GCM with a per-session SessionKey generated via CSPRNG.
- **Heap Protection**: The MicroQuickJS heap is gated with `mprotect(PROT_NONE)` between execution windows.
- **Zero-Trust Sidecar**: WebView has no filesystem access and is treated as a potentially compromised process.

## Native Capabilities
Native extensions (SQLite, Process management) are registered via a unified C binding table, ensuring consistent availability in both engines.

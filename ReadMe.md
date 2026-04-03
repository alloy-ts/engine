# AlloyScript

AlloyScript is a high-performance, dual-engine JavaScript runtime designed for security and native capabilities. It leverages **MicroQuickJS** for secure, isolated logic execution and a secondary browser engine (WebView or Native) for browser APIs, all orchestrated by a hardened C host.

## 🏗️ Architecture

AlloyScript uses a unique **Dual-Engine** architecture:

1.  **Core Logic (MicroQuickJS)**: A fast, minimal JS engine that executes all security-critical logic. It is isolated from the browser environment.
2.  **Native Bridge (WebView/Native)**: A secondary process used *only* to provide native browser APIs. It is treated as inherently hostile.
3.  **Secure IPC (E2EE)**: All communication between the C host and the secondary engine is protected by **End-to-End Encryption (AES-GCM)**.
4.  **Unified Abstraction (`AlloyJSEngine`)**: A unified vtable interface that allows the host to manage different JS engines (MQJS, WebView, Wasm) through a consistent JSON boundary.
5.  **Secure Evaluation**: `secureEval` executes code exclusively in the MicroQuickJS runtime, isolated from the UI layer.

## 🐚 Alloy Shell (`$`)

Alloy Shell provides a bash-like scripting API with a **hostile-OS threat model**. It uses a three-layer isolation hierarchy:

*   **Layer 0 (Host)**: Trusted C runtime verifying binary integrity via BLAKE3.
*   **Layer 1 (Wasm)**: WasmEdge with WASI/WIT capability-based security.
*   **Layer 2 (OCI)**: Ephemeral OCI containers (Chainguard images) for full Linux kernel isolation.

## 🛡️ Security

By default, the runtime replaces the browser's `eval` with a more restricted and secure version using MicroQuickJS. The host process is a hardened static PIE binary with `mlock` session keys and `mprotect` heap protection.

## 🚀 Key Features

1.  **Type Safety**: Structural and nominal typing with compile-time checks and no `undefined`/`null` surprises.
2.  **Multi-Deployment**: Compile once and deploy as native AOT binaries, JIT bytecode, or WASM modules.
3.  **OCI Isolation**: Every shell capability invocation runs in its own dedicated, ephemeral OCI container.
4.  **Platform Agnostic**: Uses `alloy:*` abstractions instead of platform-specific APIs, ensuring portable source code.
5.  **Web3 Ready**: Native support for decentralized identity (DID) and cryptographic verification.

## 🛠️ Building & Testing

*   **Build**: `bun run build` - Bundles TS source and prepares the C host.
*   **Test**: `bun test` - Runs the comprehensive property-based and unit test suite.

---

For more technical details, implementation snippets, and deep dives into the execution flow, see [ARCHITECTURE.md](./ARCHITECTURE.md).

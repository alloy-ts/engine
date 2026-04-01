# AlloyScript

AlloyScript is a high-performance, dual-engine JavaScript runtime designed for security and native capabilities. It leverages **MicroQuickJS** for secure, isolated logic execution and **WebView** for native browser APIs and UI, all orchestrated by a secure C host.

## Architecture

AlloyScript uses a unique **Dual-Engine** architecture:

1.  **Core Logic (MicroQuickJS)**: A fast, minimal JS engine that executes all security-critical logic. It is isolated from the browser environment.
2.  **Native Bridge (WebView)**: An unsafe WebView process used *only* to provide native browser APIs. It is treated as inherently hostile.
3.  **Secure IPC (E2EE)**: All communication between the C host and the WebView is protected by **End-to-End Encryption (AES-GCM)**.
4.  **Unified Bindings**: `alloy_bind_global` allows C/C++ functions to be exposed to both environments seamlessly.
5.  **Secure Evaluation**: `secureEval` executes code exclusively in the MicroQuickJS runtime, isolated from the UI layer.
5.  **SQLite Driver**: A high-performance driver with transactions, prepared statement caching, and `bigint` support.
6.  **Native GUI Framework (`alloy:gui`)**: A declarative component framework (ASX) that wraps native OS controls (Win32/Cocoa/GTK) using the Yoga layout engine.

## Security

By default, the runtime replaces the browser's `eval` with a more restricted and secure version using MicroQuickJS. The original `eval` is renamed to `_forbidden_eval` to discourage its use.

## Building

Use `bun run build` to bundle the TS source and prepare the C host for compilation.
The build script generates `build/bundle.c`, which contains the bundled JS source as a C string.

## Testing

Run tests with `bun test`. The test suite uses Bun's native features to mock the WebView environment and verify API behavior.

## Introduction

AlloyScript compiles a subset of TypeScript directly to native machine code, and produce a binary executable. It is a security-first local-first cross-platform application development platform runtime compiled (AoT + JIT), statically-typed programming language that is syntactacally compatible with your favarite ECMAScript/TypeScript, and a high-performance, cross-platform  application development. It combines the developer experience of TypeScript with the secure isolation of WebAssembly runtime (), and the performance of naer-native machine code.

```text
├─ Syntax: ECMAScript/TypeScript-based with AlloyScript extensions
├─ Standard Library: alloy:* modules (gui, fs, crypto, net, etc.)
├─ Target Platforms: Desktop, mobile, cloud, embedded
└─ Key Feature: Single source → multiple deployment targets
```

## Key Features

1.  **Type Safety**: Structural and nominal typing with compile-time checks and no `undefined`/`null` surprises.
2.  **Multi-Deployment**: Compile once and deploy as native AOT binaries and, JIT bytecode, or WASM modules.
3.  **Comprehensive Standard Library (`alloy:*`)**: Built-in modules for GUI, filesystem, cryptography (DID-ready), networking, and graphics.
4.  **Platform Agnostic**: Uses `alloy:*` abstractions instead of platform-specific APIs (DOM or Node.js), ensuring portable source code.
5.  **Web3 Ready**: Native support for decentralized identity (DID), content addressing (IPFS), and cryptographic verification.

## 🏗️ Architecture Overview

AlloyScript uses a layered architecture to ensure security, portability, and performance.

### Component Hierarchy

```text
┌─────────────────────────────────────┐
│ Security Sandbox Container (OCI)    │ ← OS-level isolation
├─────────────────────────────────────┤
│ App Host Orchestrator (Process)     │ ← Process lifecycle & orchestration
├─────────────────────────────────────┤
│ AlloyScript Engine (Runtime)        │ ← Code execution (WASM/JIT/AOT)
├─────────────────────────────────────┤
│ AlloyScriptCore (Abstraction)       │ ← WASI/WIT interface & capabilities
├─────────────────────────────────────┤
│ Sidecars (Optional Services)        │ ← e.g., AlloyWindow (GUI)
└─────────────────────────────────────┘
```

### Core Components

-   **App Host Orchestrator**: The main process that manages the execution environment. It owns the OS process, instantiates the engine, provides the WASI implementation, and manages the lifecycle of the application.
-   **AlloyScript Engine (ASE)**: The runtime responsible for executing AlloyScript code. It can operate in JIT mode (using WasmEdge and QuickJS) or run pre-compiled AOT native code.
-   **AlloyScriptCore**: The sandboxed abstraction layer. It defines the interface (WASI/WIT) through which the application interacts with the host, ensuring code cannot escape its sandbox.
-   **AlloyScriptContainer**: An optional OCI-compliant container (e.g., Podman/Docker) that provides an additional layer of OS-level isolation (namespaces, cgroups) around the host process.

## 🛡️ Security Model: Double Sandboxing

For maximum security, AlloyScript supports a **Double Sandboxing** architecture:

1.  **WASM Isolation (Bytecode-level)**: The `AlloyScriptEngine` executes code in a WASM sandbox, providing linear memory isolation and restricted imports via WASI/WIT.
2.  **Container Isolation (OS-level)**: The `AlloyScriptHost` runs inside an OCI container, which restricts filesystem access, networking, and system calls via Linux kernel features (namespaces, seccomp).

## 🔌 IPC & Sidecars

AlloyScript offloads complex operations like GUI rendering to **Sidecars**.

-   **AlloyWindow**: A separate process that handles native UI rendering.
-   **Communication**: The Engine (Client) communicates with Sidecars (Servers) using high-performance IPC (Unix sockets, Named pipes) via a structured RPC protocol.

## 🎯 Deployment Targets

| App Type | Has GUI? | Target | Environment |
| :--- | :--- | :--- | :--- |
| **Desktop App** | Yes | WASM (AOT binary / Bytecode) | Windows, macOS, Ubuntu |
| **Mobile App** | Yes | WASM (AOT binary / Bytecode)  | Android, iOS, |
| **CLI Tool** | No | WASM (AOT binary / Bytecode) | Terminal / Shell |
| **Web Service** | No | WASM (AOT binary / Bytecode)  | OCI-Container , Kubernetes |
| **Serverless** | No | WASM | Cloudflare Workers, Lambda |
| **Browser App** | Yes | WASM | Web Browser (via WASI/Canvas) |

---

For more technical details, implementation snippets, and deep dives into the execution flow, see [ARCHITECTURE.md](./ARCHITECTURE.md).

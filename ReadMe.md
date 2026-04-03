# AlloyScript

AlloyScript is a dual-engine runtime providing a high-security execution environment for TypeScript and JSX.

## Architecture

AlloyScript utilizes a unique **dual-engine** approach:
1. **MicroQuickJS (The Secure Core)**: A minimal, highly-embedded JS engine that executes all user logic. It is restricted to a strict ES5 subset for maximum security and predictability.
2. **WebView (The Sidecar)**: A separate, untrusted process that provides modern Browser APIs (fetch, crypto, storage) to the core via an encrypted IPC bridge.

## Architecture

- **TypeScript Native**: First-class support for TS and JSX via a professional transpilation pipeline.
- **E2EE IPC**: Inter-process communication is protected by AES-256-GCM with session keys generated per-run.
- **Hardened Host**: The native C host implements heap protection (mprotect), stack canaries, and position-independent execution.
- **Unified Bindings**: Register native functions once and have them available in both engines.
- **Cooperative Event Loop**: A robust FIFO event queue for predictable hydration of asynchronous events into MicroQuickJS.

## Getting Started

### Installation
```bash
npm install -g @alloyscript/cli
```

### Building a Project
```bash
alloy build src/main.ts -o my_app
```

### Interactive REPL
```bash
alloy repl
```

## License
- AlloyScript Source: CC0 (Public Domain)
- MicroQuickJS & WebView: MIT

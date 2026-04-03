# Alloy Shell (`$`)

Alloy Shell is a high-performance, secure shell scripting environment for AlloyScript. It provides a bash-like experience while maintaining strict security boundaries via a three-layer isolation hierarchy.

## Usage

The shell is available via the global `$` template tag.

```typescript
import { $ } from "@alloyscript/engine";

// Basic execution
await $`echo "Hello, Alloy!"`;

// Capturing output
const text = await $`ls -l`.text();

// Interpolation (safe from injection)
const name = "user; rm -rf /";
await $`echo "Hello, ${name}"`; // Prints: Hello, user; rm -rf /

// Redirection
await $`echo "data" > output.txt`;

// Pipelines
const count = await $`cat large_file.txt | grep "search" | wc -l`.text();
```

## Security Model: Three-Layer Isolation

Alloy Shell does not use the system shell (`/bin/sh`). It uses a custom interpreter compiled to WebAssembly.

1.  **Layer 0 (Host)**: The trusted C host manages the execution and verifies the integrity of the shell binary using **BLAKE3**.
2.  **Layer 1 (Wasm)**: Execution happens inside **WasmEdge** with strict **WASI/WIT** capability grants. The shell can only access files and environment variables explicitly granted by the host.
3.  **Layer 2 (OCI)**: For local-first execution, shell capabilities run in dedicated, ephemeral **OCI containers** (using Chainguard images) to ensure full kernel-level isolation.

## Builtin Commands

Alloy Shell includes 17 built-in commands that run directly in the Wasm module without spawning subprocesses:

*   **Filesystem**: `ls`, `cd`, `pwd`, `rm`, `cat`, `touch`, `mkdir`, `mv`, `dirname`, `basename`
*   **Utilities**: `echo`, `which`, `exit`, `true`, `false`, `yes`, `seq`

## Node.js Compatibility

Each shell expression has access to a simulated `process` object, allowing many existing Node.js scripts to run unmodified:

*   `process.env`: Scoped to granted environment variables.
*   `process.cwd()` / `process.chdir()`: Scoped to granted directories.
*   `process.platform` / `process.arch` / `process.version`.
*   `process.stdout` / `process.stderr` / `process.stdin`.

## Configuration

You can configure shell behavior in `alloyconfig.toml`:

```toml
[shell]
nodeEnv = "production"
disableWasmEdge = false # Set to true to use native isolation instead of Wasm
```

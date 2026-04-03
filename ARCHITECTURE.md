# AlloyScript Architecture: Technical Deep Dive

AlloyScript implements a defense-in-depth security model using a dual-engine design, encrypted inter-process communication, and a multi-layered shell isolation subsystem.

## 🏗️ The Host Orchestrator (C/C++)

The `AlloyScriptHost` is the trusted core of the system. It manages the lifecycle of all execution environments and enforces security boundaries.

### JS Engine Abstraction (`AlloyJSEngine`)

To support multiple execution targets (Native, Browser, Wasm), the host uses a unified `AlloyJSEngine` vtable. This decouples the runtime logic from specific engine implementations (MicroQuickJS, WebView, or WasmEdge).

```c
struct AlloyJSEngine {
    int (*call_fn)(AlloyJSEngine *engine, const char *name, const char *args_json, char **out_result);
    int (*eval_fn)(AlloyJSEngine *engine, const char *source, const char *filename, char **out_result);
    int (*bind_fn)(AlloyJSEngine *engine, const char *name, AlloyBindHandler handler, void *opaque);
    int (*emit_fn)(AlloyJSEngine *engine, const char *event_name, const char *payload_json);
    void (*destroy_fn)(AlloyJSEngine *engine);
};
```

### Dual-Engine Design

- **MicroQuickJS**: The primary execution engine for application logic. It runs in the same process as the host, providing high performance and direct access to secure C bindings.
- **WebView**: A secondary, untrusted engine used for DOM/Browser APIs. It is treated as a hostile environment.

### Secure IPC (E2EE)

AlloyScript implements **End-to-End Encryption** for all communication between the C host and the WebView. At startup, a unique 256-bit AES session key is generated and pinned in RAM (`mlock`).

1. **MQJS -> WebView**: Requests are serialized to JSON, encrypted via AES-256-GCM in the host, and sent to the WebView sidecar.
2. **WebView -> MQJS**: Results are decrypted, authenticated, and validated against a strict schema before being returned to user code.

---

## 🐚 Alloy Shell: Three-Layer Isolation

The Alloy Shell (`$`) operates under a **hostile OS threat model**. It assumes the underlying OS might be compromised and uses three layers of isolation for local-first execution.

### Isolation Hierarchy

```text
Layer 0: HostProcess (AlloyScript C runtime — trusted)
  │
  ├── Layer 1: WasmEdge + WASI/WIT
  │     └── shell.wasm (Bytecode verified before execution)
  │           │
  │           └── Layer 2: OCI Container (Linux Kernel Isolation)
  │                 └── Default image: cgr.dev/chainguard/alpine-base
  │                 └── Signature verification via Sigstore/cosign
```

### WIT Interface (`alloy:shell`)

The `alloy:shell` WIT interface is the canonical contract for all shell operations. It defines the `exec` function and associated options/results.

```wit
interface shell {
    record shell-opts { throws: bool, env: list<tuple<string, string>>, cwd: string }
    exec: func(script: string, opts: shell-opts) -> result<shell-result, string>;
}
```

### Enforcement Mechanisms

| Mode | Enforcement | Primitives |
| :--- | :--- | :--- |
| **Local (Wasm)** | WasmEdge | WASI capability grants (`--dir`, `--env`). |
| **Local (Native)**| Host Kernel | Namespaces (PID, Mount, Net), Seccomp-BPF, Seatbelt (macOS). |
| **Server** | Host C Runtime | Whitelist hashtable enforcement. |

---

## 🛡️ Double Sandboxing Execution Flow

The security of AlloyScript relies on two independent layers of isolation for every capability.

```text
┌─────────────────────────────────────────────┐
│ 1. AlloyScript Host (C Orchestrator)        │
│    - Verifies binary integrity (BLAKE3)     │
│    - Spawns isolated processes/containers   │
│    - Manages AES-GCM session keys           │
└──────────────────┬──────────────────────────┘
                   ↓
┌─────────────────────────────────────────────┐
│ 2. OCI Container (Layer 2 Sandbox)          │
│    - Isolated PID, Network, and FS namespaces│
│    - Resource limits (cgroups)              │
│    - Chainguard minimal base images         │
└──────────────────┬──────────────────────────┘
                   ↓
┌─────────────────────────────────────────────┐
│ 3. WasmEdge + WASI (Layer 1 Sandbox)        │
│    - Linear memory isolation                │
│    - Capability-based security              │
│    - WIT boundary enforcement               │
└──────────────────┬──────────────────────────┘
                   ↓
┌─────────────────────────────────────────────┐
│ 4. Shell Interpreter (shell.wasm)           │
│    - Safe string interpolation (no-injection)│
│    - Cross-platform builtins                │
└─────────────────────────────────────────────┘
```

### Binary Self-Attestation

The HostProcess verifies its own integrity at startup using **BLAKE3**. It reads its own binary, zeros the embedded hash field, and compares the result against a pre-computed hash stored in `.rodata`.

## 📦 Container Environment

In `local-first` mode, every capability invocation runs in a fresh, ephemeral OCI container. Images are verified using **Sigstore/cosign** keyless verification before start.

- **Backend**: `apple/container` on macOS/iOS, `podman` on Linux/Windows.
- **Images**: Prefer verified Chainguard images (`cgr.dev/chainguard/...`).

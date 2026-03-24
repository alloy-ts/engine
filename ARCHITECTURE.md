# AlloyScript Architecture: Technical Deep Dive

This document provides a detailed technical overview of the AlloyScript architecture, focusing on the host orchestrator, the execution environment, and the security model.

## 🏗️ The Host Orchestrator (Rust)

The `AlloyScriptHost` is a lightweight orchestrator implemented in Rust. It is responsible for managing the lifecycle of the application and enforcing capability-based security.

### Orchestrator Implementation Sketch

```rust
// Simple orchestrator that manages Podman containers and RPC dispatching

pub struct AlloyScriptHost {
    podman_client: PodmanClient,
    ipc_server: IpcServer,
    capability_manager: CapabilityManager,
}

impl AlloyScriptHost {
    pub async fn run_app(&self, app_bundle: &Path) -> Result<()> {
        let config = AppConfig::load(app_bundle)?;

        // Create Podman container with resource limits and restricted capabilities
        let container = self.podman_client.create_container(
            PodmanCreateOptions {
                image: "localhost/alloyos:latest",
                name: config.app_id.clone(),
                memory: config.memory_limit.unwrap_or(512 * 1024 * 1024),
                cpus: config.cpu_limit.unwrap_or(1.0),
                cap_drop: vec!["ALL"],
                security_opt: vec!["no-new-privileges=true".into()],
                mounts: vec![
                    Mount { source: app_bundle.display().to_string(), target: "/app", read_only: true },
                    Mount { source: format!("/tmp/alloy-{}", config.app_id), target: "/data", read_only: false },
                ],
                ..Default::default()
            },
        ).await?;

        self.podman_client.start(&container).await?;
        Ok(())
    }
}
```

## 🛡️ Double Sandboxing Execution Flow

The security of AlloyScript relies on two independent layers of isolation.

```text
┌─────────────────────────────────────────────┐
│ 1. AlloyScript Host (Rust Orchestrator)     │
│    - Parses application bundle              │
│    - Spawns OCI container via Podman        │
│    - Listens for RPC calls via Unix socket  │
└──────────────────┬──────────────────────────┘
                   ↓
┌─────────────────────────────────────────────┐
│ 2. OCI Container (OS-level Sandbox)         │
│    - Isolated PID, Network, and FS namespaces│
│    - Resource limits (cgroups)              │
│    - System call filtering (seccomp)        │
└──────────────────┬──────────────────────────┘
                   ↓
┌─────────────────────────────────────────────┐
│ 3. WasmEdge + QuickJS (WASM Sandbox)        │
│    - Linear memory isolation                │
│    - No direct syscalls (WASI/WIT only)     │
│    - Capability-based imports               │
└──────────────────┬──────────────────────────┘
                   ↓
┌─────────────────────────────────────────────┐
│ 4. Application (Bytecode)                   │
│    - Executes in memory-safe environment    │
│    - Interaction requires RPC via Host      │
└─────────────────────────────────────────────┘
```

### Sandbox Responsibilities

| Layer | Enforced By | Isolation Features |
| :--- | :--- | :--- |
| **OS Sandbox** | Linux Kernel / Podman | PID namespaces, Network namespaces, Chroot/OverlayFS, cgroups, seccomp filters. |
| **WASM Sandbox**| WasmEdge / WIT | Linear memory, Type safety, Disallowance of arbitrary pointer arithmetic, Import/Export control. |

## 📦 Container Environment (Dockerfile)

The base image for AlloyScript applications is designed to be minimal and secure.

```dockerfile
# Dockerfile: AlloyOS base image (Hardened)

FROM cgr.dev/chainguard/base:latest

# Install WasmEdge and dependencies
RUN apk add --no-cache ca-certificates libssl3 libstdc++ glibc
RUN curl -sSf https://raw.githubusercontent.com/WasmEdge/WasmEdge/master/utils/install.sh | bash -s -- -v 0.13.5

# Copy runtime components
COPY qjs-wasi.wasm /usr/local/lib/wasmedge/

# Entry point executes the application via WasmEdge
ENTRYPOINT ["/usr/local/bin/wasmedge"]
CMD ["/app/app.wasm"]
```

## 🔌 RPC Dispatcher Logic

When an application needs to access host services (like the GUI), it makes an RPC call. The Host validates the application's capabilities before forwarding the request.

```rust
impl IpcDispatcher {
    pub async fn dispatch(&self, request: RpcRequest) -> Result<RpcResponse> {
        match request.method.as_str() {
            "alloy:gui/create-window" => {
                if !self.has_capability(Capability::GuiAccess) {
                    return Err("GUI access denied".into());
                }
                self.forward_to_alloy_gui(request).await
            }
            "alloy:net/connect-tcp" => {
                // Check fine-grained network capabilities (hosts, ports)
                self.validate_network_access(&request).await?;
                self.forward_to_network_service(request).await
            }
            _ => Err(format!("Unknown method: {}", request.method).into()),
        }
    }
}
```

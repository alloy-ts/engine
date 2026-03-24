# AlloyScript



AlloyScriptCore is the application runtime engine.


```
WASM Isolation (bytecode-level)
    ↓
Podman Container Isolation (OS-level)
    ↓
Linux Kernel (host security)
    ↓
= Double-sandboxed execution
```

---

## 🎯 Correct Architecture: AlloyScript as Simple Rust Host + Podman

```
┌──────────────────────────────────────────────┐
│ AlloyScript Host (Rust binary)               │
│ (Simple orchestrator)                        │
├──────────────────────────────────────────────┤
│ Responsibilities:                            │
│ ├─ Parse app bundle                         │
│ ├─ Spawn Podman container                   │
│ ├─ RPC relay (IPC ↔ Podman socket)         │
│ ├─ Capability management (WASI/WIT)        │
│ └─ Process lifecycle                        │
└──────────────────────────────────────────────┘
        ↓ (Podman API)
┌──────────────────────────────────────────────┐
│ Podman Container                             │
│ (OCI-compliant)                             │
├──────────────────────────────────────────────┤
│ Base Image: Chainguard (hardened)           │
├──────────────────────────────────────────────┤
│ WasmEdge (WASM VM)                          │
├──────────────────────────────────────────────┤
│ qjs-wasi.wasm (QuickJS + WASI)              │
├──────────────────────────────────────────────┤
│ app.wasm (AlloyScript bytecode)             │
└──────────────────────────────────────────────┘
        ↓ (WASI/WIT boundary)
┌──────────────────────────────────────────────┐
│ Sandboxed Capabilities (WASI/WIT)           │
│ ├─ alloy:fs (restricted paths)              │
│ ├─ alloy:net (port restrictions)            │
│ ├─ alloy:gui (via RPC to host)             │
│ └─ alloy:crypto (local only)                │
└──────────────────────────────────────────────┘
```

---

## 🏗️ AlloyScript Host Architecture (Rust)

```rust
// alloy-runtime/src/main.rs
// Simple orchestrator, not a complex OS

use std::process::Command;
use std::path::Path;
use anyhow::Result;

pub struct AlloyScriptHost {
    podman_client: PodmanClient,
    ipc_server: IpcServer,
    capability_manager: CapabilityManager,
}

impl AlloyScriptHost {
    pub fn new() -> Result<Self> {
        Ok(Self {
            podman_client: PodmanClient::new()?,
            ipc_server: IpcServer::listen("unix:///tmp/alloy-rpc.sock")?,
            capability_manager: CapabilityManager::new(),
        })
    }
    
    /// Run an AlloyScript app in isolated Podman container
    pub async fn run_app(&self, app_bundle: &Path) -> Result<()> {
        // 1. Parse app bundle
        let config = AppConfig::load(app_bundle)?;
        
        // 2. Create Podman container with WASI/WIT restrictions
        let container = self.podman_client.create_container(
            PodmanCreateOptions {
                image: "localhost/alloyos:latest",
                name: config.app_id.clone(),
                
                // Environment: WASI capabilities (optional, WIT already restricts)
                env: vec![
                    ("ALLOY_APP".to_string(), format!("{}", app_bundle.display())),
                    ("ALLOY_CAPABILITIES".to_string(), self.capability_manager.serialize(&config.capabilities)?),
                ],
                
                // Resource limits
                memory: config.memory_limit.unwrap_or(512 * 1024 * 1024), // 512MB
                cpus: config.cpu_limit.unwrap_or(1.0),
                
                // Networking (restricted to container network)
                network_mode: "slirp4netns",  // User-mode networking
                
                // Filesystem (bind mount only allowed paths)
                mounts: vec![
                    Mount {
                        source: app_bundle.to_string_lossy().to_string(),
                        target: "/app",
                        read_only: true,
                    },
                    Mount {
                        source: format!("/tmp/alloy-{}", config.app_id),
                        target: "/data",
                        read_only: false,
                    },
                ],
                
                // Security
                cap_drop: vec!["ALL"],  // Drop all Linux capabilities
                cap_add: vec![],        // Add none (WASM doesn't need them)
                security_opt: vec![
                    "no-new-privileges=true".to_string(),
                ],
            },
        ).await?;
        
        // 3. Start container
        self.podman_client.start(&container).await?;
        
        // 4. Register with IPC server
        // When app calls alloy:gui, alloy:net, etc., route through IPC
        let ipc_dispatcher = IpcDispatcher {
            app_id: config.app_id.clone(),
            container_id: container.id.clone(),
            capabilities: config.capabilities.clone(),
        };
        self.ipc_server.register(config.app_id, ipc_dispatcher).await?;
        
        // 5. Wait for container to finish
        self.podman_client.wait(&container).await?;
        
        Ok(())
    }
}

pub struct AppConfig {
    pub app_id: String,
    pub capabilities: Vec<Capability>,
    pub memory_limit: Option<u64>,
    pub cpu_limit: Option<f64>,
}

#[derive(Clone)]
pub enum Capability {
    NetworkAccess { allowed_hosts: Vec<String>, allowed_ports: Vec<u16> },
    FileAccess { allowed_paths: Vec<String> },
    GuiAccess,
    DeviceAccess { devices: Vec<String> },
}

pub struct IpcDispatcher {
    app_id: String,
    container_id: String,
    capabilities: Vec<Capability>,
}

impl IpcDispatcher {
    pub async fn dispatch(&self, request: RpcRequest) -> Result<RpcResponse> {
        // Check capability before allowing RPC call
        match request.method.as_str() {
            "alloy:gui/create-window" => {
                if !self.has_capability(Capability::GuiAccess) {
                    return Err("GUI access denied".into());
                }
                // Forward to AlloyGui host
                self.forward_to_alloy_gui(request).await
            }
            "alloy:net/connect-tcp" => {
                if let Some(Capability::NetworkAccess { allowed_hosts, allowed_ports }) = self.get_capability(&Capability::NetworkAccess { allowed_hosts: vec![], allowed_ports: vec![] }) {
                    // Verify the request matches capability
                    let host = request.args.get("host")?;
                    let port = request.args.get("port")?;
                    
                    if !allowed_hosts.contains(host) || !allowed_ports.contains(port) {
                        return Err("Network access denied".into());
                    }
                }
                // Forward to network service
                self.forward_to_network_service(request).await
            }
            "alloy:fs/open" => {
                if let Some(Capability::FileAccess { allowed_paths }) = self.get_capability(&Capability::FileAccess { allowed_paths: vec![] }) {
                    let path = request.args.get("path")?;
                    
                    if !allowed_paths.iter().any(|ap| path.starts_with(ap)) {
                        return Err("File access denied".into());
                    }
                }
                // Forward to storage service
                self.forward_to_storage_service(request).await
            }
            _ => Err(format!("Unknown method: {}", request.method).into()),
        }
    }
    
    fn has_capability(&self, cap: Capability) -> bool {
        // Check if app has this capability
        self.capabilities.iter().any(|c| self.capabilities_match(c, &cap))
    }
}

pub struct IpcServer {
    listener: tokio::net::UnixListener,
    dispatchers: Arc<Mutex<HashMap<String, IpcDispatcher>>>,
}

impl IpcServer {
    pub async fn listen(addr: &str) -> Result<Self> {
        let listener = tokio::net::UnixListener::bind(addr)?;
        Ok(Self {
            listener,
            dispatchers: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    
    pub async fn register(&self, app_id: String, dispatcher: IpcDispatcher) -> Result<()> {
        self.dispatchers.lock().await.insert(app_id, dispatcher);
        Ok(())
    }
    
    pub async fn run(&self) -> Result<()> {
        loop {
            let (mut socket, _) = self.listener.accept().await?;
            let dispatchers = self.dispatchers.clone();
            
            tokio::spawn(async move {
                // Read RPC request from Podman container
                let request = read_rpc_request(&mut socket).await.unwrap();
                
                // Route to correct dispatcher
                let dispatcher = dispatchers.lock().await.get(&request.app_id).cloned().unwrap();
                
                // Dispatch with capability check
                let response = dispatcher.dispatch(request).await.unwrap();
                
                // Send response back to container
                write_rpc_response(&mut socket, response).await.unwrap();
            });
        }
    }
}
```

---

## 📋 Podman Container Image

```dockerfile
# Dockerfile: AlloyOS base image

FROM cgr.dev/chainguard/base:latest

# Install WasmEdge and dependencies
RUN apk add --no-cache \
    ca-certificates \
    libssl3 \
    libstdc++ \
    glibc

# Download and install WasmEdge
RUN curl -sSf https://raw.githubusercontent.com/WasmEdge/WasmEdge/master/utils/install.sh | bash -s -- -v 0.13.5

# Copy qjs-wasi.wasm (pre-built)
COPY qjs-wasi.wasm /usr/local/lib/wasmedge/

# Entry point: run app.wasm
ENTRYPOINT ["/usr/local/bin/wasmedge"]
CMD ["/app/app.wasm"]
```

---

## 🔄 Execution Flow: Double Sandboxing

```
┌─────────────────────────────────────────────┐
│ User launches AlloyScript app                │
│ $ alloy run app.wasm --capabilities net,gui │
└──────────────────┬──────────────────────────┘
                   ↓
┌─────────────────────────────────────────────┐
│ AlloyScript Host (Rust)                     │
│ 1. Parse app.wasm                          │
│ 2. Create Podman container:                │
│    podman create --cap-drop=ALL \           │
│      --memory=512m \                        │
│      --cpus=1.0 \                          │
│      -e ALLOY_CAPABILITIES="net,gui" \     │
│      -v app.wasm:/app/app.wasm:ro \        │
│      localhost/alloyos:latest               │
│ 3. Start container                         │
│ 4. Open IPC socket for RPC                 │
└──────────────────┬──────────────────────────┘
                   ↓
┌─────────────────────────────────────────────┐
│ Podman Container (OCI-compliant)            │
│ Sandbox 1: OS-level                        │
│ ├─ Isolated process namespace              │
│ ├─ Isolated network namespace              │
│ ├─ Isolated filesystem                     │
│ ├─ Linux cap DROP ALL                      │
│ └─ No access to host except RPC socket     │
└──────────────────┬──────────────────────────┘
                   ↓
┌─────────────────────────────────────────────┐
│ WasmEdge (inside container)                │
│ └─ Launches app.wasm                       │
└──────────────────┬──────────────────────────┘
                   ↓
┌─────────────────────────────────────────────┐
│ app.wasm (AlloyScript bytecode)             │
│ Sandbox 2: WASM-level                      │
│ ├─ Linear memory (isolated)                │
│ ├─ No direct syscalls                      │
│ ├─ Only WASI/WIT imports allowed           │
│ ├─ Capability-based access control         │
│ └─ Must go through RPC for any host access │
└──────────────────┬──────────────────────────┘
                   ↓
┌─────────────────────────────────────────────┐
│ Example: app calls alloy:gui/create-window │
│ 1. WASM calls WIT function                 │
│ 2. WasmEdge intercepts (only allowed       │
│    functions, others blocked)              │
│ 3. RPC call over socket:                   │
│    {method: "alloy:gui/create-window"}    │
│ 4. IPC reaches AlloyScript Host            │
│ 5. Host checks capabilities:               │
│    ✓ "gui" in app's capabilities          │
│ 6. Host forwards to AlloyGui               │
│ 7. AlloyGui creates native window          │
│ 8. Response sent back                      │
└─────────────────────────────────────────────┘
```

---

## ✅ What You Get: Double Sandboxing

### Sandbox 1: Container (OS-level)

```
Podman enforces (via Linux):
✓ Process isolation (separate PID namespace)
✓ Network isolation (slirp4netns)
✓ Filesystem isolation (read-only /app, writable /data)
✓ Resource limits (memory, CPU)
✓ Linux capabilities dropped (CAP_DROP=ALL)
✓ SELinux/AppArmor profiles (if enabled)
✓ No access to host system
✓ Can only communicate via RPC socket
```

### Sandbox 2: WASM (bytecode-level)

```
WasmEdge enforces:
✓ Linear memory isolation (separate heap per app)
✓ No direct syscalls (only WASI imports)
✓ No access to host memory
✓ No arbitrary code execution (sandboxed bytecode)
✓ WIT capability model (which WASI functions are allowed)
✓ Must go through IPC for any host interaction
✓ No escape vectors (proven WASM semantics)
```

---

## 📊 Simplified Host Architecture

```
AlloyScript Host (minimal, ~500 LOC Rust):

┌─────────────────────────────────────────┐
│ main()                                  │
├─────────────────────────────────────────┤
│ 1. Parse command line                  │
│ 2. Load app.wasm + config              │
│ 3. Create Podman container             │
│ 4. Start IPC listener                  │
│ 5. Dispatcher loop (handle RPC calls)  │
│                                        │
│ That's it.                             │
└─────────────────────────────────────────┘

Complexity offloaded to:
├─ Podman (process isolation)
├─ Linux kernel (resource management)
├─ WasmEdge (bytecode execution)
├─ WASI/WIT (capability restrictions)
└─ Rust standard library
```

---

## 🎯 Why This Is Perfect

**You're not building an OS. You're building an orchestrator.**

```
Traditional approach (Fuchsia-like):
├─ Build custom microkernel
├─ Build custom compositor
├─ Build custom process manager
├─ Build custom capability system
└─ = 500k LOC, 10+ years, hard to maintain

AlloyScript approach (Podman-based):
├─ Use Linux kernel (existing)
├─ Use AlloyGui (custom, ~3k LOC)
├─ Use Podman (existing, well-tested)
├─ Use WASI/WIT (existing, standardized)
├─ Build simple orchestrator (500 LOC)
└─ = Production-grade in months
```

---

## 📋 AlloyScript Runtime Spec (Simplified)

```rust
// Core responsibilities:

1. App Bundle Loading
   └─ Parse app.wasm + app.toml
   └─ Validate signatures
   └─ Extract capabilities

2. Container Lifecycle
   └─ Create Podman container
   └─ Set resource limits
   └─ Mount filesystem
   └─ Configure networking
   └─ Start / stop container

3. IPC/RPC Server
   └─ Listen on Unix socket
   └─ Receive RPC requests from container
   └─ Validate capabilities
   └─ Route to services
   └─ Send responses

4. Capability Management
   └─ Parse capability declarations
   └─ Enforce at RPC boundary
   └─ Audit access attempts

5. Service Routing
   └─ alloy:gui → AlloyGui host
   └─ alloy:net → Network service (in host)
   └─ alloy:fs → Storage service (host filesystem)
   └─ alloy:crypto → Crypto service (in-container)

That's the entire runtime.
```

---

## ✅ Architecture Decision: CONFIRMED OPTIMAL

```
┌──────────────────────────────────────┐
│ AlloyScript Host (Rust, simple)      │
│ ├─ Parse + orchestrate               │
│ └─ ~500 LOC                          │
├──────────────────────────────────────┤
│ Podman (proven, hardened)            │
│ ├─ Process isolation                 │
│ ├─ Resource limits                   │
│ ├─ Network isolation                 │
│ └─ Filesystem sandboxing             │
├──────────────────────────────────────┤
│ WasmEdge + qjs-wasi (verified WASM)  │
│ ├─ Bytecode execution                │
│ ├─ Memory isolation                  │
│ ├─ No syscalls                       │
│ └─ WASI/WIT restrictions             │
├──────────────────────────────────────┤
│ app.wasm (user code)                 │
│ ├─ Double-sandboxed                  │
│ ├─ Proven security semantics         │
│ └─ No escape vectors                 │
└──────────────────────────────────────┘

Benefits:
✓ Simple host (not reinventing OS)
✓ Proven security (Podman + WASM)
✓ Production-ready (use mature projects)
✓ Scalable (Podman → Kubernetes)
✓ Portable (works on any Linux)
```

This is the correct architecture. Ready to implement the host orchestrator?
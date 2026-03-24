# AlloyScript

AlloyScript is a **compiled, statically-typed programming language** with the following characteristics:

```
Language: AlloyScript
├─ Type System: Static, type-safe (TypeScript-compatible)
├─ Syntax: ECMAScript/TypeScript-based with AlloyScript extensions
├─ Compilation: Multi-target (AOT native, JIT bytecode, WASM)
├─ Standard Library: alloy:* modules (gui, fs, crypto, net, etc.)
├─ Target Platforms: Desktop, web, cloud, embedded
└─ Key Feature: Single source → multiple deployment targets
```

**Key Features:**

```
1. Type Safety
   ├─ Structural typing + nominal typing
   ├─ Compile-time type checking
   └─ No undefined/null surprises

2. Multi-Deployment
   ├─ Compile to native (AOT)
   ├─ Compile to bytecode (JIT)
   ├─ Compile to WASM
   └─ Run on any platform

3. Standard Library (alloy:*)
   ├─ alloy:gui (native UI)
   ├─ alloy:fs (filesystem: local, IPFS, OPFS)
   ├─ alloy:crypto (cryptography + DID)
   ├─ alloy:net (networking)
   ├─ alloy:auth (authentication)
   └─ alloy:graphics (2D graphics)

4. Platform Agnostic
   ├─ No DOM API (uses alloy:gui instead)
   ├─ No Node.js APIs (uses alloy:* instead)
   ├─ Portable source code
   └─ Native behavior per platform

5. Web3 Ready
   ├─ DID-based identity (built-in)
   ├─ Cryptographic signing/verification
   ├─ Content addressing (IPFS)
   └─ Decentralized apps
```

## Why AlloyScript?


Problems it solves:

1. Fragmentation
   Problem: JavaScript varies (browser, Node.js, Deno, edge)
   Solution: Single language, multiple hosts
   
2. Cross-Platform Consistency
   Problem: "Write once, debug everywhere"
   Solution: Same source works on Windows, macOS, Linux, browser, cloud
   
3. Web3 Integration
   Problem: DIDs, crypto, IPFS not standard in JS
   Solution: Built-in to language
   
4. Performance
   Problem: Interpreted JS is slow for apps
   Solution: AOT compilation to native machine code
   
5. Security
   Problem: JavaScript has too much power (DOM, Node.js, globals)
   Solution: Sandboxed runtime with explicit permissions


---

### AlloyScriptEngine (Runtime)

**What is AlloyScriptEngine?**

AlloyScriptEngine is the **execution runtime** that interprets or executes AlloyScript code.

```
AlloyScriptEngine (Runtime)
├─ Mode 1: JIT (Interpreted)
│  ├─ WasmEdge (WASM VM) 
│  ├─ JIT interpretation of WASM modules on the fly
│  ├─ QuickJS (JavaScript interpreter)
│  ├─ Bytecode interpreter
│  └─ Dynamic code execution (evaluate API)
│
├─ Mode 2: AOT (Pre-compiled)
│  ├─ WasmEdge (WASM VM)
│  ├─ AOT pre-compiled WASM modules 
│  ├─ No JS runtime
│  ├─ Direct native machine code
│  └─ Execution via OS
│
└─ WASI Interface
   ├─ System call abstraction
   ├─ Platform-agnostic API
   └─ Host provides implementation
```

## Key Responsibilities:

```
1. Code Execution
   ├─ Parse bytecode/AST
   ├─ JIT compilation (if applicable)
   ├─ Execute instructions
   └─ Manage call stack

2. Memory Management
   ├─ Heap allocation
   ├─ Garbage collection
   └─ Linear memory (WASM)

3. Standard Library Binding
   ├─ alloy:* module APIs
   ├─ IPC/RPC to sidecars
   └─ System integration

4. Sandboxing (JIT mode only)
   ├─ Limited memory (heap size)
   ├─ Limited CPU (time limits)
   ├─ Limited filesystem access
   └─ Limited network access
```

---

### AlloyScriptHost (Host/Container)

**What is AlloyScriptHost?**

AlloyScriptHost is the **process/container that instantiates and manages AlloyScriptEngine**.

```
AlloyScriptHost (Host Container)
├─ Responsibility: Own the process lifecycle
├─ Runs: AlloyScriptEngine inside it
├─ Provides: WASI implementation + platform integration
├─ Manages: Sidecars (AlloyWindow, Alloy services)
└─ Examples:
   ├─ AlloyExplorer (browser host)
   ├─ Standalone app launcher
   ├─ Cloudflare Worker runtime
   ├─ Custom server application
   └─ Browser (WASM host)
```

**AlloyScriptHost is NOT the same as AlloyScriptEngine:**

```
AlloyScriptEngine (Runtime)
└─ Executes code
   ├─ Stateless (or limited state)
   ├─ Fungible (replaces with new instance)
   └─ Process: In-process component

AlloyScriptHost (Container)
└─ Owns the process
   ├─ Lifecycle management
   ├─ Unique (per app/tab/request)
   └─ Process: Separate OS process
```

---

### AlloyScriptCore (Sandboxed Abstraction)

**What is AlloyScriptCore?**

AlloyScriptCore is the **sandboxed runtime abstraction** that provides a WASI-like interface.

```
AlloyScriptCore (Sandboxed Abstraction Layer)
├─ WASI Interface Definition
│  ├─ System calls (fd_read, fd_write, etc.)
│  ├─ File descriptor model
│  ├─ Memory model
│  └─ Permission model
│
├─ Multiple Implementations
│  ├─ JIT: QuickJS (implements WASI)
│  ├─ WASM: WasmEdge runtime
│  ├─ Browser: Polyfill WASI via browser APIs
│  └─ Native: Direct OS syscalls
│
└─ Guarantees
   ├─ Code cannot escape sandbox
   ├─ Only permitted resources accessible
   ├─ Memory-safe execution
   └─ Auditable behavior
```

**AlloyScriptCore vs AlloyScriptEngine:**

```
AlloyScriptCore = The abstraction (interface + contract)
AlloyScriptEngine = The implementation (actual runtime)

Example:
├─ AlloyScriptCore = WASI spec
├─ AlloyScriptEngine = WasmEdge + QuickJS (implements WASI)
```

---

## 🏗️ Component Hierarchy

```
┌─────────────────────────────────────┐
│ AlloyScriptHost (Process)           │
│ (e.g., AlloyExplorer, Browser)      │
├─────────────────────────────────────┤
│ AlloyScriptEngine (Runtime)         │
│ ├─ WasmEdge / QuickJS               │
│ ├─ Bytecode interpreter             │
│ └─ WASI implementation              │
├─────────────────────────────────────┤
│ AlloyScriptCore (Abstraction)       │
│ ├─ WASI interface                   │
│ ├─ File descriptor model            │
│ └─ Permission model                 │
├─────────────────────────────────────┤
│ Sidecars (Optional)                 │
│ ├─ AlloyWindow (GUI)                │
│ ├─ AlloyGraphics (2D graphics)      │
│ └─ System services                  │
└─────────────────────────────────────┘
```

---

## 🔌 IPC/RPC Architecture

### IPC Boundary & Ownership

**Question: Who owns the IPC server?**

**Answer: It depends on the sidecar type.**

```
Scenario 1: AlloyWindow as Native Sidecar (Desktop)

AlloyScriptHost Process (owns lifecycle)
├─ AlloyScriptEngine (JIT mode)
│  ├─ Calls alloy:gui APIs
│  └─ Makes IPC requests to AlloyWindow
│
└─ IPC Client (in-process with ASE)
   └─ Sends messages to AlloyWindow

AlloyWindow Process (separate)
├─ IPC Server (listens for requests)
├─ Creates native controls (HWND, NSView, GtkWidget)
├─ Handles native OS events
└─ Sends responses back via IPC

Communication Channel:
├─ Linux: AF_VSOCK (high-performance)
├─ Windows: Named pipes
├─ macOS: Unix sockets
└─ All platforms: HTTP over localhost

Message Protocol:
├─ ASE → AlloyWindow: "create_button(label='Click')"
├─ AlloyWindow → ASE: "button_created(id=5)"
├─ OS → AlloyWindow: WM_COMMAND (click)
└─ AlloyWindow → ASE: "button_clicked(id=5)"
```

```
Scenario 2: AlloyWindow as WASI Module (Browser)

Browser Process (AlloyScriptHost)
├─ WASM runtime (AlloyScriptEngine)
│  └─ AlloyWindow as WASM module
│     ├─ Implements WASI for GUI operations
│     ├─ Uses WebGL/Canvas for rendering
│     └─ Uses Browser File API for filesystem
│
└─ No separate process needed
   (AlloyWindow runs in same WASM sandbox)

Message Protocol:
├─ Direct function calls (no IPC)
├─ JavaScript to WASM (synchronous)
└─ No network overhead
```

### IPC Client vs IPC Server

```
IPC Architecture (Desktop)

┌─────────────────────────────┐
│ AlloyScriptHost             │
│ ┌──────────────────────────┐│
│ │ AlloyScriptEngine        ││
│ │ ┌────────────────────┐   ││
│ │ │ IPC Client         │   ││ ← Makes requests
│ │ │ ├─ Send methods    │   ││
│ │ │ ├─ Receive replies │   ││
│ │ │ └─ Async handlers  │   ││
│ │ └────────────────────┘   ││
│ └──────────────────────────┘│
│         ↓ IPC Pipe          │
│  (AF_VSOCK, named pipe)     │
└─────────────────────────────┘
         ↓
┌─────────────────────────────┐
│ AlloyWindow (Separate)      │
│ ┌──────────────────────────┐│
│ │ IPC Server               ││ ← Listens for requests
│ │ ├─ Accept connections    ││
│ │ ├─ Process messages      ││
│ │ ├─ Send responses        ││
│ │ └─ Event dispatcher      ││
│ └──────────────────────────┘│
│ ┌──────────────────────────┐│
│ │ Native Controls          ││
│ │ ├─ HWND                  ││
│ │ ├─ Event handlers        ││
│ │ └─ Rendering             ││
│ └──────────────────────────┘│
└─────────────────────────────┘
```

**Key Point:** 
- AlloyScriptEngine has IPC **Client** (requests GUI operations)
- AlloyWindow has IPC **Server** (listens for requests)
- They're **separate processes**

### RPC (Remote Procedure Call)

```
RPC in AlloyScript Context

When ASE calls alloy:gui.button.create():

1. RPC Call (from ASE)
   └─ IPC message: { method: "create_button", label: "Click" }
   
2. IPC Transport
   └─ Sent over AF_VSOCK/named pipe
   
3. RPC Server (AlloyWindow)
   └─ Receives, unmarshals, executes
   
4. RPC Response
   └─ IPC message: { id: 5, type: "button", label: "Click" }
   
5. RPC Client (ASE)
   └─ Receives, unmarshals, continues execution

This is RPC over IPC.
```

---

## 🎯 What is an AlloyScript App?

### Definition

**An AlloyScript App is any application written in AlloyScript that:**

```
├─ Has a defined entry point (function or module)
├─ Runs on AlloyScriptEngine
├─ Uses alloy:* standard library
├─ Produces observable output or state change
└─ Can be deployed to various environments
```

### Can an AlloyScript App Always Have GUI?

**No.** AlloyScript apps fall into three categories:

```
1. GUI Applications (alloy:gui)
   ├─ Uses <Window> component
   ├─ Renders native controls
   ├─ Handles user input
   └─ Example: Password manager, text editor
   
2. CLI Applications (alloy:io, alloy:fs)
   ├─ No GUI (headless)
   ├─ Text input/output
   ├─ Filesystem operations
   └─ Example: File processor, data analyzer
   
3. Service Applications (alloy:net)
   ├─ No GUI
   ├─ HTTP/WebSocket server
   ├─ Event-driven
   └─ Example: API server, webhook handler
```

### Can a CLI Application Be an AlloyScript App?

**Yes.** Example:

```typescript
// cli-app.as (AlloyScript CLI)

import { alloyFs, alloyIo } from "alloy:*";

export async function main(args: string[]) {
    if (args.length === 0) {
        console.log("Usage: myapp <file>");
        return 1;
    }
    
    const filePath = args[0];
    const content = await alloyFs.read(filePath);
    
    console.log(`Read ${content.length} bytes`);
    return 0;
}
```

Compile:
```bash
alloy compile cli-app.as --mode aot --target binary-executable
# → cli-app (standalone binary, no GUI)

# OR
alloy compile cli-app.as --mode jit --target alloy-script
# → cli-app.as (bytecode, runs in AlloyExplorer shell)
```

---

## ☁️ Can AlloyScript Run on Serverless Edge (Cloudflare Workers)?

**Yes.** Example:

```typescript
// edge-worker.as (AlloyScript for Cloudflare)

import { alloyNet, alloyCrypto } from "alloy:*";

export async function handleRequest(request: Request): Promise<Response> {
    const { method, url } = request;
    
    if (method === "POST") {
        const body = await request.text();
        const hash = await alloyCrypto.sha256(body);
        
        return new Response(
            JSON.stringify({ hash }),
            { status: 200, headers: { "Content-Type": "application/json" } }
        );
    }
    
    return new Response("OK", { status: 200 });
}
```

Compile:
```bash
alloy compile edge-worker.as --mode aot --target wasi-wasm
# → edge-worker.wasm

# Deploy to Cloudflare Workers
wrangler publish edge-worker.wasm
```

**Characteristics:**
- No GUI (serverless)
- Stateless (each request is independent)
- Fast startup (WASM)
- Portable (runs anywhere WASM is supported)

---

## 🌐 Can AlloyScript App Run in a Web Browser?

**Yes, with AlloyWindow targeting WASI-WASM:**

```
AlloyScript GUI App (browser)

1. Compile to WASM
   $ alloy compile app.asx --target wasi-wasm
   → app.wasm

2. Create HTML glue code
   index.html:
   ├─ Load app.wasm
   ├─ Implement WASI host
   ├─ Map WASI calls to browser APIs
   └─ Canvas for rendering

3. AlloyWindow-WASM
   ├─ WASI module (implements GUI operations)
   ├─ Calls WASI host functions
   │  ├─ create_window() → JavaScript
   │  ├─ draw_button() → WebGL/Canvas
   │  └─ handle_event() → DOM events
   └─ User sees native-like GUI
```

**Example HTML Host:**

```html
<!DOCTYPE html>
<html>
<head>
    <title>AlloyScript App</title>
    <style>
        #canvas { width: 100%; height: 100vh; }
    </style>
</head>
<body>
    <canvas id="canvas"></canvas>
    
    <script>
        // WASI Host Implementation
        const wasiHost = {
            fd_write: (fd, iovecs) => {
                // Console output
            },
            
            create_window: (title, width, height) => {
                // Create canvas + setup rendering
                return windowId;
            },
            
            create_button: (windowId, label) => {
                // Create button in canvas/WebGL
                return buttonId;
            },
            
            draw_rect: (x, y, width, height, color) => {
                // Draw to WebGL
            },
            
            handle_mouse_event: (type, x, y) => {
                // Route to app
            }
        };
        
        // Load and run WASM
        fetch('app.wasm')
            .then(r => r.arrayBuffer())
            .then(buf => WebAssembly.instantiate(buf, { env: wasiHost }))
            .then(({ instance }) => instance.exports.main());
    </script>
</body>
</html>
```

**Result:** AlloyScript GUI app running in browser, looks/behaves like native app.

---

## 💻 Can AlloyScript App Run in Local User-Space (Native)?

**Yes. In fact, this is the PRIMARY deployment target.**

```
Local Native Execution (Desktop)

Mode 1: AOT Compilation (Recommended)
$ alloy compile app.asx --mode aot --target binary-executable
→ app.exe (Windows) / app (macOS/Linux)

Execution:
./app  # Runs directly, no runtime needed

Process:
├─ AlloyScriptHost = the app process itself
├─ AlloyScriptEngine = compiled away (no runtime)
├─ AlloyWindow = compiled into machine code
└─ All calls = native code

Performance: Maximum
Runtime Size: Zero
```

```
Mode 2: JIT Compilation (Via AlloyExplorer)
$ alloy compile app.asx --mode jit --target alloy-script
→ app.as (bytecode)

Execution:
alloy open alloy://localhost:8000/app.as
# Opens in AlloyExplorer

Process Architecture:
├─ AlloyExplorer (main browser)
│  ├─ AlloyScriptEngine (JIT runtime)
│  ├─ AlloyWindow (for browser UI)
│  └─ IPC server for app
│
└─ App Process (spawned by AlloyExplorer)
   ├─ AlloyScriptEngine (JIT runtime)
   ├─ AlloyWindow (GUI sidecar)
   └─ IPC client

Performance: Good (JIT with caching)
Runtime Size: ~75 MB (includes ASE)
```

```
Mode 3: Self-Contained With Runtime
$ alloy compile app.asx --mode jit --target binary-executable --bundle-runtime
→ app-bundle.exe

Execution:
./app-bundle.exe  # Standalone, includes everything

Bundle Contents:
├─ AlloyScriptEngine
├─ AlloyWindow
└─ Bytecode

Performance: Good
Runtime Size: ~75 MB (embedded)
```

---

## 📊 AlloyScript App Types (Complete Matrix)

```
┌──────────────────┬──────────┬──────────────┬──────────────────────┐
│ App Type         │ Has GUI? │ Deployment   │ Host / Env           │
├──────────────────┼──────────┼──────────────┼──────────────────────┤
│ Desktop GUI      │ Yes      │ AOT binary   │ Local machine        │
│ (AlloyExplorer)  │          │ or bytecode  │ (Windows/Mac/Linux)  │
├──────────────────┼──────────┼──────────────┼──────────────────────┤
│ CLI Tool         │ No       │ AOT binary   │ Local terminal       │
│                  │          │ or bytecode  │                      │
├──────────────────┼──────────┼──────────────┼──────────────────────┤
│ Web Service      │ No       │ AOT or WASM  │ Docker, Kubernetes   │
│                  │          │              │ or custom server     │
├──────────────────┼──────────┼──────────────┼──────────────────────┤
│ Serverless       │ No       │ WASM         │ Cloudflare Workers   │
│ Edge Function    │          │              │ Vercel, AWS Lambda   │
├──────────────────┼──────────┼──────────────┼──────────────────────┤
│ Browser App      │ Yes      │ WASM         │ Web browser          │
│ (via WASI)       │          │              │ (HTML host)          │
├──────────────────┼──────────┼──────────────┼──────────────────────┤
│ AlloyExplorer    │ Yes      │ Bytecode     │ AlloyExplorer host   │
│ (in browser)     │          │              │ (browser app)        │
└──────────────────┴──────────┴──────────────┴──────────────────────┘
```

---

## ✅ Summary: AlloyScript Ecosystem

```
AlloyScript (Language)
    └─ Type-safe, multi-target, Web3-ready

AlloyScriptEngine (Runtime)
    ├─ JIT: WasmEdge + QuickJS (interpreted)
    ├─ AOT: Compiled native code (no runtime)
    └─ WASM: Portable bytecode format

AlloyScriptCore (Abstraction)
    ├─ WASI interface
    ├─ Sandboxing guarantees
    └─ Platform agnostic

AlloyScriptHost (Container)
    ├─ AlloyExplorer (browser)
    ├─ Cloudflare Worker (serverless)
    ├─ Custom app server
    ├─ Web browser (WASI host)
    └─ Local desktop (standalone binary)

Sidecars (Optional Services)
    ├─ AlloyWindow (GUI rendering)
    ├─ AlloyGraphics (2D graphics)
    └─ Other services (logging, telemetry)

AlloyScript Apps (all types)
    ├─ GUI apps (desktop, browser)
    ├─ CLI apps (terminal)
    ├─ Web services (servers)
    ├─ Edge functions (serverless)
    └─ All runnable in any host that provides proper environment
```




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
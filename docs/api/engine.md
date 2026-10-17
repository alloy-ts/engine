# Engine API Design & Documentation

`@alloy-ts/engine` provides native JavaScript execution backends via Node-API (NAPI).
It exposes `Engine` handles that wrap execution runtimes and manage contexts, scripts, memory limits, and module loading.

## What is `boa_engine`?

`boa_engine` is an ECMAScript engine written in Rust implementing ECMAScript standard library builtins, bytecompiler, virtual machine, and AST optimizer.

Key modules in `boa_engine`:
- `context`: Runtime context and state.
- `realm`: Specification realm record (intrinsics, global object, environments).
- `vm`: Bytecode interpreter and call frames.
- `builtins`: Standard builtins (`Array`, `Object`, `Promise`, `Math`, `JSON`, etc.).
- `script`: Script parsing and evaluation.
- `module`: ES module loaders and module graphs.

## Multi-Backend Architecture: `Engine`

Our NAPI bindings support both **Boa** (pure Rust ECMAScript engine) and **QuickJS** execution backends through `Engine`.

### Factory Constructors

```ts
import { Engine } from "@alloy-ts/engine";

// Default Engine instance (QuickJS / Boa dual backend capabilities)
const engine = new Engine();

// Explicit factory constructors:
const quickjsEngine = Engine.quickjs();
const boaEngine = Engine.boa();
```

### Context Creation

From an `Engine` instance, you can create isolated execution contexts:

```ts
// Create a boa-backed Context
const ctx = engine.createContext();
ctx.eval("1 + 2"); // 3

// Create a QuickJS-backed Context (for Atom bindings)
const qjsCtx = engine.createQuickjsContext();
qjsCtx.eval("'hello' + ' world'"); // "hello world"
```

### Memory & Execution Controls

- `engine.setMemoryLimit(limitBytes: number)` — sets maximum memory ceiling.
- `engine.setMaxStackSize(stackBytes: number)` — sets execution stack limit.
- `engine.setGcThreshold(thresholdBytes: number)` — sets memory threshold for GC triggers.
- `engine.runGc()` — triggers manual garbage collection.
- `engine.memoryUsage()` — returns detailed memory allocation statistics object.
- `engine.executePendingJob(): boolean` — executes next pending promise/job callback in queue.

### Weak Reference Handles

- `engine.weak()` — returns a `WeakEngine` reference that does not prevent garbage collection of the underlying runtime. Call `.tryRef()` to promote back to strong handle if still alive.

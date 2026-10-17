# Engine API Documentation

The `Engine` (`JsEngine`) in `@alloy-ts/engine` provides an embedded JavaScript execution engine wrapping Boa (`boa_engine`), QuickJS (`rquickjs`), and the host Node-API environment (`napi::Env`).

---

## 1. Overview of `boa_engine` API Capabilities

The `boa_engine` Rust crate implements an ECMAScript-compliant virtual machine, bytecompiler, runtime context, and standard library builtins:

- **Context & Execution**: `boa_engine::Context` (evaluates source code via `eval()`, manages realms, registers global properties/callables/classes, drains promise jobs).
- **Realm Record**: `boa_engine::realm::Realm` (isolated global object, intrinsics, global scope environment, template maps, and loaded module maps).
- **Scripts & Bytecompiler**: `boa_engine::script::Script` and `boa_engine::bytecompiler::ByteCompiler` (compiles source AST down to Boa VM bytecode `CodeBlock`s).
- **Values & Objects**: `boa_engine::JsValue` and `boa_engine::object::JsObject` (prototype chains, property descriptors, proxies, typed arrays).
- **Garbage Collector**: `boa_gc::Gc` (traceable reference-counted GC pointers for ECMAScript memory management).
- **Modules**: `boa_engine::module::Module` and `boa_engine::module::ModuleLoader` (ES module loading and resolution).

---

## 2. NAPI Engine Architecture & Design

`@alloy-ts/engine` exposes the `Engine` (`JsEngine`) class through NAPI bindings:

### Dual Engine Backends

1. **Boa Engine (Default)**:
   - Primary ECMAScript runtime for `Context` evaluation, script execution, realm management, and host function bindings.
   - Instantiated via `new Engine()` or `new Context()`.
2. **QuickJS Engine (`Engine.quickjs()`)**:
   - Secondary lightweight C-based engine integration via `rquickjs`.
   - Used by `Atom` and `QjsContext` for high-performance low-level AST manipulation, bytecode execution, and memory statistics inspection.

```ts
import { Engine } from "@alloy-ts/engine";

// Default Boa-backed Engine
const engine = new Engine();

// QuickJS-backed Engine handle
const qjsContext = engine.createQuickjsContext();
```

---

## 3. `Engine` JS API Reference

### Construction & Backends

```ts
// Create a new Engine instance (optionally passing a Loader)
const engine = new Engine(loader?: Loader);

// Check if host environment/addon has been disposed
engine.isDisposed(): boolean;
```

### Context Factory Methods

```ts
// Create a new Boa-backed execution Context
const ctx = engine.createContext(); // -> Context

// Create a new QuickJS-backed execution Context
const qjsCtx = engine.createQuickjsContext(); // -> QjsContext
```

### Memory & Runtime Tuning

```ts
// Set max memory limit in bytes (0 = unlimited)
engine.setMemoryLimit(bytes: number): void;

// Set max stack size limit in bytes (default: 256 KB)
engine.setMaxStackSize(bytes: number): void;

// Set GC threshold in bytes
engine.setGcThreshold(bytes: number): void;

// Set engine informational string
engine.setInfo(info: string): void;

// Force a garbage collection sweep
engine.runGc(): void;

// Read memory usage statistics
const stats = engine.memoryUsage();
```

### Promise Job Draining

```ts
// Check if microtask jobs are pending
if (engine.isJobPending()) {
  // Execute pending jobs one by one
  while (engine.executePendingJob()) {}
}
```

### Weak Engine Handles

```ts
// Get a weak reference to the engine that does not prevent GC
const weak = engine.weak();

// Attempt to re-acquire strong handle (returns null if freed)
const strong = weak.tryRef(); // -> Engine | null
```

### Host N-API Environment Utilities

```ts
// Create host JS values and inspect host global scope
const arr = engine.createArray(10);
const str = engine.createString("hello");
const num = engine.createDouble(3.14);
const hasMath = engine.hasGlobal("Math"); // true
const napiVer = engine.getNapiVersion();

// Host instance data persistence
engine.setInstanceData("session-data");
const data = engine.getInstanceData(); // "session-data"
```

---

## 4. TypeScript Usage Example

```ts
import { Engine, Loader } from "@alloy-ts/engine";

// 1. Initialize Loader and Engine
const loader = new Loader();
loader.add("math", "export const add = (a, b) => a + b;");

const engine = new Engine(loader);

// 2. Create isolated Contexts
const ctx = engine.createContext();
ctx.eval('import { add } from "math"; globalThis.val = add(10, 20);');
console.log(ctx.eval("globalThis.val")); // 30

// 3. Inspect Memory Usage & Force GC
console.log(engine.memoryUsage().memory_used_size);
engine.runGc();

// 4. Drain Jobs
while (engine.isJobPending()) {
  engine.executePendingJob();
}
```

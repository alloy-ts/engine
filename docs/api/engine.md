# Engine API Documentation

The `Engine` class (`JsEngine`) provides a unified native N-API bridge to JavaScript execution runtimes. It encapsulates both the embedded **Boa Engine** (`boa_engine`) and **QuickJS** (`rquickjs`) engines alongside host N-API capabilities.

---

## What is `boa_engine`?

`boa_engine` is Boa's core Rust crate implementing ECMAScript standard library built-in objects, a JavaScript bytecode compiler, and a virtual machine for code execution.

### `boa_engine` Core Rust Submodules

- **`context::Context`**: Execution context manager (realms, globals, jobs, limits).
- **`realm::Realm`**: ECMAScript Realm Record management.
- **`script::Script`**: Bytecode parser and evaluator.
- **`object::*`**: `JsObject`, `ObjectInitializer`, `FunctionObjectBuilder`.
- **`value::JsValue`**: Dynamic ECMAScript value variant (Number, String, Boolean, Object, Symbol, BigInt, Undefined, Null).
- **`builtins::*`**: Standard built-ins (`Object`, `Array`, `Function`, `Promise`, `Math`, `JSON`, `Reflect`, `Proxy`, `Console`, etc.).
- **`module::*`**: ES Module loaders, resolvers, and graph evaluation.
- **`optimizer::*`**: Bytecode AST optimization passes (constant folding, dead code elimination).
- **`vm::*`**: Virtual Machine call frames, stack management, and runtime limits.

---

## NAPI JS `Engine` Export Design

The `Engine` NAPI export exposes multi-backend support through `Engine.backend` (`boa` vs `quickjs`), providing native context instantiation for both engines.

```ts
import { Engine, Loader } from "@alloy-ts/engine";

// Backend identifiers map
const backends = Engine.backend(); // { boa: "boa", quickjs: "quickjs" }

// Default / Boa Engine instance
const boaEngine = Engine.boa();
const ctx = boaEngine.createContext();
ctx.eval("1 + 1"); // 2

// QuickJS Engine instance
const loader = new Loader();
const qjsEngine = Engine.quickjs(loader);
const qjsCtx = qjsEngine.createQuickjsContext();
qjsCtx.eval("2 + 2"); // 4
```

---

## API Reference

### Static Methods

| Method | Signature | Description |
| --- | --- | --- |
| `Engine.backend()` | `Engine.backend(): { boa: string; quickjs: string }` | Returns backend constant map `{ boa: "boa", quickjs: "quickjs" }`. |
| `Engine.boa(loader?)` | `Engine.boa(loader?: Loader): Engine` | Creates a Boa-backed `Engine` instance. |
| `Engine.quickjs(loader?)` | `Engine.quickjs(loader?: Loader): Engine` | Creates a QuickJS-backed `Engine` instance. |
| `new Engine(loader?)` | `new Engine(loader?: Loader)` | Default constructor (creates `Engine` instance). |

### Context Creation

| Method | Signature | Returns | Description |
| --- | --- | --- | --- |
| `createContext()` | `createContext(): Context` | `Context` | Creates a new Boa-backed execution context. |
| `createQuickjsContext()` | `createQuickjsContext(): QjsContext` | `QjsContext` | Creates a new QuickJS-backed context (for use with `Atom`). |

### Memory & Garbage Collection

| Method | Signature | Description |
| --- | --- | --- |
| `setMemoryLimit(bytes)` | `setMemoryLimit(limit: number): void` | Sets max memory limit in bytes (`0` = unlimited). |
| `setMaxStackSize(bytes)` | `setMaxStackSize(limit: number): void` | Sets maximum execution stack size in bytes. |
| `setGcThreshold(bytes)` | `setGcThreshold(threshold: number): void` | Sets memory allocation threshold that triggers GC. |
| `runGc()` | `runGc(): void` | Manually triggers garbage collection. |
| `memoryUsage()` | `memoryUsage(): object` | Returns memory allocation statistics object (`malloc_size`, `obj_count`, etc.). |

### Job Queue & Execution

| Method | Signature | Returns | Description |
| --- | --- | --- | --- |
| `isJobPending()` | `isJobPending(): boolean` | `boolean` | Checks whether at least one promise microtask job is pending. |
| `executePendingJob()` | `executePendingJob(): boolean` | `boolean` | Executes the first pending job in the queue. |

### Handles & Host N-API Helpers

| Method | Signature | Description |
| --- | --- | --- |
| `weak()` | `weak(): WeakEngine` | Gets a weak handle to this engine (`weak.tryRef()` promotes back to `Engine`). |
| `isDisposed()` | `isDisposed(): boolean` | Checks if the engine N-API context has been disposed or unloaded. |
| `createArray(length)` | `createArray(len: number): unknown[]` | Builds a host JS array of the requested length. |
| `createString(str)` | `createString(value: string): string` | Builds a host JS string. |
| `createInt32(num)` | `createInt32(int: number): number` | Builds a host JS int32 number. |
| `createDouble(num)` | `createDouble(value: number): number` | Builds a host JS double float. |
| `makeObject()` | `makeObject(): object` | Creates a host JS object instance. |
| `setInstanceData(val)` | `setInstanceData(value: string): void` | Persists agent instance data on host N-API context. |
| `getInstanceData()` | `getInstanceData(): string \| null` | Reads back host instance data. |

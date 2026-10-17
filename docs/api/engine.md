# Engine

`Engine` (`JsEngine` in Rust) represents the underlying ECMAScript execution engine. It wraps the execution runtime (Boa engine by default, with support for QuickJS backend via `Engine.quickjs()`) and the host Node-API (`napi::Env`) environment.

## Boa Engine Architecture

The `boa_engine` crate implements an ECMAScript context, AST parser/optimizer, bytecompiler, and virtual machine for JavaScript execution in Rust:
- **`Context`**: Primary entry point for code evaluation and global scope.
- **`Realm`**: Realm Record bundling intrinsics, global environment, and per-realm caches.
- **`boa_gc`**: Traceable, garbage-collected heap pointer management (`Gc<T>`).
- **`boa_parser` & `boa_ast`**: AST generation and statement list optimization.

## JS / NAPI Bindings (`Engine`)

In JavaScript, `Engine` provides high-level control over engine configuration, context creation, memory limits, and GC.

### Static Constructors & Constants

| API | Type / Return | Description |
| --- | --- | --- |
| `new Engine(loader?)` | `Engine` | Default constructor using Boa backend. |
| `Engine.quickjs(loader?)` | `Engine` | Static factory constructor using QuickJS backend. |
| `Engine.backend` | `{ Boa: "boa", QuickJs: "quickjs" }` | Constant object exposing backend engine identifiers. |

```ts
import { Engine } from "./main.ts";

// Default engine (Boa backend)
const engine = new Engine();

// QuickJS engine constructor
const qjsEngine = Engine.quickjs();

// Backend enum/constants
console.log(Engine.backend.Boa);     // "boa"
console.log(Engine.backend.QuickJs); // "quickjs"
```

### Context Creation

| JS method | Returns | Description |
| --- | --- | --- |
| `createContext()` | `Context` | Creates a new Boa-backed execution `Context`. |
| `createQuickjsContext()` | `QjsContext` | Creates a QuickJS-backed `QjsContext` (for QuickJS `Atom` bindings). |

### Engine Control & Memory Management

| JS method | Signature | Description |
| --- | --- | --- |
| `setMemoryLimit` | `setMemoryLimit(limitBytes: number): void` | Sets max memory usage in bytes. |
| `setMaxStackSize` | `setMaxStackSize(limitBytes: number): void` | Sets max execution stack size in bytes. |
| `setGcThreshold` | `setGcThreshold(thresholdBytes: number): void` | Sets garbage collection trigger threshold. |
| `runGc` | `runGc(): void` | Manually triggers garbage collection. |
| `memoryUsage` | `memoryUsage(): object` | Returns memory usage statistics object. |
| `isJobPending` | `isJobPending(): boolean` | Returns true if pending promise jobs exist. |
| `executePendingJob` | `executePendingJob(): boolean` | Executes the next pending promise job. |
| `weak` | `weak(): WeakEngine` | Returns a weak handle to this engine. |
| `isDisposed` | `isDisposed(): boolean` | Returns true if the host environment is unloaded. |

### Host N-API Operations

| JS method | Signature | Description |
| --- | --- | --- |
| `createArray` | `createArray(length: number): unknown[]` | Allocates a JS Array on the host V8 engine. |
| `createString` | `createString(str: string): string` | Creates a JS String on the host engine. |
| `createInt32` | `createInt32(n: number): number` | Creates a JS int32 number on the host engine. |
| `createDouble` | `createDouble(n: number): number` | Creates a JS double number on the host engine. |
| `hasGlobal` | `hasGlobal(name: string): boolean` | Checks if a property exists on host `globalThis`. |
| `getNapiVersion` | `getNapiVersion(): number` | Returns the host Node-API version. |
| `makeObject` | `makeObject(): object` | Creates a sample host object. |

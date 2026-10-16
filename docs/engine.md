# Engine (`docs/engine.md`)

The `Engine` class is the primary entry point for managing the QuickJS JavaScript execution engine in `@alloy-ts/engine`. It wraps both an embedded QuickJS `Runtime` and the host Node-API (`napi::Env`) environment context.

## Overview

```ts
import { Engine, Loader } from "@alloy-ts/engine";

const engine = new Engine();
const ctx = engine.create_context();
```

## Constructor

### `new Engine(loader?: Loader)`

Creates a new QuickJS engine instance.

- **`loader`** _(optional)_: An instance of `Loader` providing custom ES module resolution and loading.

---

## Methods

### QuickJS Runtime Management

#### `engine.create_context(): Context`

Creates and returns a new execution `Context` within this engine. Multiple contexts created from the same engine can share JavaScript objects and functions.

#### `engine.set_loader(loader: Loader): void`

Attaches or replaces the module loader for this engine.

#### `engine.weak(): WeakRuntime`

Returns a weak handle to this runtime engine (`WeakRuntime`).

#### `engine.set_memory_limit(limitInBytes: number): void`

Sets a limit (in bytes) on the maximum memory allocation for QuickJS. Pass `0` for unlimited.

#### `engine.set_max_stack_size(limitInBytes: number): void`

Sets a limit (in bytes) on the maximum stack size for QuickJS. Default is `256 * 1024` (256 KB).

#### `engine.set_gc_threshold(thresholdInBytes: number): void`

Sets the memory threshold (in bytes) that triggers automatic garbage collection cycles.

#### `engine.set_info(info: string): void`

Sets an informational string on the runtime engine for debugging.

#### `engine.run_gc(): void`

Manually triggers a QuickJS garbage collection cycle.

#### `engine.memory_usage(): object`

Returns memory usage statistics as a plain JavaScript object matching QuickJS `JSMemoryUsage`.

#### `engine.is_job_pending(): boolean`

Returns `true` if there is at least one pending job (e.g. promise resolution microtask) in the job queue.

#### `engine.execute_pending_job(): boolean`

Executes the first pending job in the queue. Returns `true` if a job was executed, or `false` if the queue was empty. Throws if the job raises an unhandled exception.

#### `engine.execPendingJob(): boolean`

Alias for `execute_pending_job()`.

---

### N-API Host Utilities

#### `engine.is_disposed(): boolean`

Returns `true` if the host Node-API environment context has been unloaded.

#### `engine.create_array(length: number): unknown[]`

Allocates a JavaScript array of the specified length using the host N-API context.

#### `engine.create_string(value: string): string`

Creates a JavaScript string in the host environment.

#### `engine.create_int32(value: number): number`

Creates a 32-bit integer in the host environment.

#### `engine.create_double(value: number): number`

Creates a double-precision float in the host environment.

#### `engine.has_global(name: string): boolean`

Checks if the host global object (`globalThis`) has a property named `name`.

#### `engine.get_napi_version(): number`

Returns the Node-API version supported by the host runtime.

#### `engine.make_object(): object`

Creates a new JavaScript object via N-API with default metadata properties (`{ ok: true, engine: "napi" }`).

#### `engine.set_instance_data(data: string): void`

Persists VM-specific state string on the host Node-API agent instance.

#### `engine.get_instance_data(): string | null`

Reads back persisted VM-specific state string.

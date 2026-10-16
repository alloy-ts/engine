# Engine API

The `Engine` class wraps both the Node-API context (`napi::Env`) and an embedded QuickJS `Runtime`.

## Overview

```ts
import { Engine } from "@alloy-ts/engine";

const engine = new Engine();
```

## Methods

### QuickJS Runtime Capabilities

- **`create_context(): Context`**
  Creates a new QuickJS execution context sharing this engine's runtime.

- **`set_loader(loader: Loader): void`**
  Configures module resolution and loading on the runtime using a `Loader`.

- **`set_memory_limit(limit: number): void`**
  Sets the maximum memory limit (in bytes) for the QuickJS engine.

- **`set_max_stack_size(limit: number): void`**
  Sets the maximum stack size (in bytes) for the engine (default: 256KB).

- **`set_gc_threshold(threshold: number): void`**
  Sets the memory threshold (in bytes) that triggers garbage collection.

- **`run_gc(): void`**
  Manually triggers garbage collection.

- **`memory_usage(): object`**
  Returns memory usage statistics matching QuickJS `JSMemoryUsage`.

- **`is_job_pending(): boolean`**
  Returns whether at least one promise job is pending in the runtime queue.

- **`execute_pending_job(): boolean` / `execPendingJob(): boolean`**
  Executes the first pending promise job in the runtime queue. Returns `true` if a job ran.

### Host N-API Context Capabilities

- **`create_string(value: string): unknown`** — Builds a JS string via N-API.
- **`create_int32(int: number): unknown`** — Builds a JS int32 number via N-API.
- **`create_double(value: number): unknown`** — Builds a JS double number via N-API.
- **`create_array(len: number): unknown[]`** — Allocates a JS array via N-API.
- **`has_global(name: string): boolean`** — Checks if property `name` exists on `globalThis`.
- **`get_napi_version(): number`** — Returns host N-API version.
- **`is_disposed(): boolean`** — Returns whether the addon instance is unloaded.

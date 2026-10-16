# Engine

`Engine` is a JavaScript-visible N-API class that provides access to Node-API context methods as well as QuickJS runtime operations.

## Overview

When instantiated in a Node.js / NAPI environment, `Engine` automatically captures the host `NapiEngine` context while also initializing an embedded QuickJS `Runtime`.

## Usage

```ts
import { Engine } from "@alloy-ts/engine";

const engine = new Engine();

// Node-API Context Operations
engine.create_string("hello"); // "hello"
engine.create_int32(42); // 42
engine.create_array(3); // [ <3 empty items> ]
engine.has_global("Math"); // true
engine.get_napi_version(); // host N-API version

// QuickJS Runtime Operations
engine.is_job_pending(); // false
engine.execPendingJob(); // false
engine.execute_pending_job(); // false
engine.run_gc(); // triggers QuickJS GC
engine.set_memory_limit(1024 * 1024);
engine.set_max_stack_size(256 * 1024);
```

## Methods

- **`is_disposed(): boolean`**: Returns `true` if the underlying N-API context has been unloaded.
- **`create_array(len: number): unknown[]`**: Creates a JavaScript array of the requested length.
- **`create_string(value: string): string`**: Creates a JavaScript string.
- **`create_int32(int: number): number`**: Creates a JavaScript number from an `i32`.
- **`create_double(value: number): number`**: Creates a JavaScript number from an `f64`.
- **`has_global(name: string): boolean`**: Checks if a named property exists on `globalThis`.
- **`get_napi_version(): number`**: Returns the host runtime N-API version.
- **`make_object(): object`**: Builds an object via the N-API context.
- **`set_instance_data(value: string): void`**: Persists VM-specific state on the agent.
- **`get_instance_data(): string | null`**: Reads back the persisted state.
- **`run_gc(): void`**: Triggers QuickJS garbage collection.
- **`set_memory_limit(limit: number): void`**: Sets QuickJS memory limit in bytes.
- **`set_max_stack_size(limit: number): void`**: Sets QuickJS max stack size in bytes.
- **`is_job_pending(): boolean`**: Checks for pending QuickJS async jobs.
- **`execute_pending_job(): boolean` / `execPendingJob(): boolean`**: Executes the first pending job in QuickJS.

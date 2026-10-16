# Engine

The `Engine` class represents an embedded JavaScript execution engine powered by [QuickJS](https://bellard.org/quickjs/) (`rquickjs`) safe high-level bindings, combined with direct access to the host Node-API environment (`napi::Env`).

## Overview

An `Engine` instance provides two unified facets on a single object:

1. **Embedded QuickJS Runtime**: Owns QuickJS runtime state, creates isolated contexts (`Context`), configures memory and stack limits, manages garbage collection, and drains promise/microtask job queues.
2. **Host N-API Environment**: Interacts with the host environment (Node.js / V8) using the `napi::Env` captured at construction time.

## Constructor

```ts
import { Engine, Loader } from "@alloy-ts/engine";

// Create a bare engine
const engine = new Engine();

// Or create an engine seeded with a module loader
const loader = new Loader();
const engineWithLoader = new Engine(loader);
```

## API Reference

### QuickJS Runtime Facet

#### `engine.create_context(): Context`

Creates a new QuickJS realm (`Context`) sharing this engine. Contexts created from the same engine share runtime settings and module loaders, but maintain independent global scopes (`globalThis`).

#### `engine.set_loader(loader: Loader): void`

Attaches or replaces the ES module loader for this engine.

#### `engine.weak(): WeakEngine`

Returns a weak reference to the engine (`WeakEngine`). Calling `weak.try_ref()` returns an `Engine` if still alive, or `null` if garbage collected.

#### `engine.set_memory_limit(limitInBytes: number): void`

Sets the maximum memory limit (in bytes) for the QuickJS runtime. Pass `0` for unlimited.

#### `engine.set_max_stack_size(limitInBytes: number): void`

Sets the maximum call stack size in bytes. (Default is `256 * 1024`).

#### `engine.set_gc_threshold(thresholdInBytes: number): void`

Sets the memory allocation threshold that triggers garbage collection.

#### `engine.set_info(info: string): void`

Sets an informational string on the runtime for debugging purposes.

#### `engine.run_gc(): void`

Manually triggers a QuickJS garbage collection sweep.

#### `engine.memory_usage(): object`

Returns memory statistics from QuickJS (such as `malloc_size`, `memory_used_size`, `atom_count`, `obj_count`, etc.).

#### `engine.is_job_pending(): boolean`

Returns `true` if there are pending jobs (microtasks/promise reactions) queued in the engine.

#### `engine.execute_pending_job(): boolean` / `engine.execPendingJob(): boolean` / `engine.exec_pending_job(): boolean`

Executes the next pending microtask/job in the queue.
Returns `true` if a job ran, or `false` if the job queue was empty.
Throws if a queued job (e.g., `queueMicrotask`) throws an error.

### Host N-API Environment Facet

#### `engine.is_disposed(): boolean`

Returns `false` as long as the underlying N-API environment remains valid.

#### `engine.create_array(length: number): unknown[]`

Creates a host JavaScript array with the given length in the host V8 runtime.

#### `engine.create_string(value: string): string`

Creates a host string value in the host V8 runtime.

#### `engine.create_int32(value: number): number`

Creates a host 32-bit integer number in the host V8 runtime.

#### `engine.create_double(value: number): number`

Creates a host double-precision floating point number in the host V8 runtime.

#### `engine.has_global(name: string): boolean`

Checks if a given property name exists on the host `globalThis`.

#### `engine.get_napi_version(): number`

Returns the Node-API version supported by the host runtime.

#### `engine.make_object(): object`

Creates a host object `{ ok: true, engine: "napi" }`.

#### `engine.set_instance_data(data: string): void`

Persists custom string data on the host N-API environment agent instance.

#### `engine.get_instance_data(): string | null`

Retrieves the string data persisted on the host N-API environment.

## Usage Example

```ts
import { Engine, Loader } from "@alloy-ts/engine";

const loader = new Loader();
loader.add("math", "export const square = (x) => x * x;");

const engine = new Engine(loader);

// Facet 1: Embedded QuickJS execution
const ctx = engine.create_context();
ctx.eval_module('import { square } from "math"; globalThis.res = square(6);');
console.log(ctx.eval("globalThis.res")); // 36

// Drain promise microtasks
if (engine.is_job_pending()) {
  engine.execPendingJob();
}

// Facet 2: Host N-API environment
console.log(engine.get_napi_version()); // e.g. 9
console.log(engine.has_global("process")); // true
```

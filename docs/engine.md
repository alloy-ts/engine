# Engine & WeakEngine Documentation

The `Engine` class combines an embedded [QuickJS](https://bellard.org/quickjs/) JavaScript runtime (via the [`rquickjs`](https://crates.io/crates/rquickjs) Rust crate) with direct access to the host Node-API (`napi::Env`) environment.

## Overview

Each `Engine` instance manages two facets:

1. **QuickJS Runtime Facet**: Mints isolated realms (`Context`), loads modules registered with a `Loader`, manages runtime limits and garbage collection, and drains promise/microtask job queues.
2. **Host N-API Environment Facet**: Reflects into the host environment (Node.js/V8), builds host-native JavaScript values, and persists host-level instance data.

Both camelCase (e.g. `execPendingJob`) and snake_case (e.g. `execute_pending_job`) method variants are supported.

---

## Constructor

### `new Engine(loader?: Loader)`

Constructs a new `Engine` instance. Captures the host N-API `Env` and initializes a fresh QuickJS runtime.

- **Parameters**:
  - `loader` _(optional)_: A [`Loader`](loader.md) instance pre-seeded with ES module sources.

---

## QuickJS Runtime Methods

### `createContext()` / `create_context(): Context`

Creates a new [`Context`](context.md) realm sharing this engine's QuickJS runtime.

### `setLoader(loader: Loader)` / `set_loader(loader: Loader): void`

Attaches or replaces the module loader for this engine.

### `isJobPending()` / `is_job_pending(): boolean`

Returns `true` if there is at least one pending job (e.g., resolved promise reaction or microtask) in the engine queue.

### `execPendingJob()` / `executePendingJob()` / `execute_pending_job(): boolean`

Executes the next pending job in the queue.

- **Returns**: `true` if a job was executed, `false` if the queue was empty.
- **Throws**: If a microtask throws an unhandled error, `execPendingJob()` re-throws that exception.

### `setMemoryLimit(limitBytes: number)` / `set_memory_limit(limitBytes: number): void`

Sets a limit (in bytes) on the maximum memory the QuickJS runtime can allocate. Pass `0` for unlimited.

### `setMaxStackSize(stackBytes: number)` / `set_max_stack_size(stackBytes: number): void`

Sets the maximum C stack size allocated for QuickJS evaluation (default: 256 KiB).

### `setGcThreshold(thresholdBytes: number)` / `set_gc_threshold(thresholdBytes: number): void`

Sets the memory threshold (in bytes) that triggers automatic garbage collection.

### `setInfo(info: string)` / `set_info(info: string): void`

Sets an informational description string for the runtime (useful for debugging tooling).

### `runGc()` / `run_gc(): void`

Triggers an immediate garbage collection sweep over the QuickJS runtime.

### `memoryUsage()` / `memory_usage(): object`

Returns an object containing memory allocation statistics for the QuickJS engine (such as `malloc_size`, `memory_used_size`, `atom_count`, `obj_count`, etc.).

### `weak(): WeakEngine`

Creates a weak handle (`WeakEngine`) to this engine.

---

## Host N-API Environment Methods

### `isDisposed()` / `is_disposed(): boolean`

Returns `true` if the underlying N-API environment has been disposed (e.g., when the native addon is unloaded).

### `createArray(length: number): unknown[]` / `create_array(length: number): unknown[]`

Creates a host JavaScript `Array` of the specified length in Node's environment.

### `createString(value: string): string` / `create_string(value: string): string`

Creates a host JavaScript string value.

### `createInt32(value: number): number` / `create_int32(value: number): number`

Creates a host 32-bit integer number value.

### `createDouble(value: number): number` / `create_double(value: number): number`

Creates a host floating-point double number value.

### `hasGlobal(name: string): boolean` / `has_global(name: string): boolean`

Checks whether a named property exists on the host global object (e.g., Node's `globalThis`).

### `getNapiVersion(): number` / `get_napi_version(): number`

Returns the Node-API version supported by the host runtime.

### `makeObject(): { ok: boolean, engine: string }` / `make_object(): { ok: boolean, engine: string }`

Creates a plain host object with default metadata.

### `setInstanceData(data: string): void` / `set_instance_data(data: string): void`

Stores host-level instance data associated with the current N-API environment.

### `getInstanceData(): string | null` / `get_instance_data(): string | null`

Reads back the stored host-level instance data.

---

## `WeakEngine`

`WeakEngine` holds a weak reference to an `Engine` without preventing it from being garbage collected.

### `tryRef()` / `try_ref(): Engine | null`

Attempts to upgrade the weak handle back to a strong `Engine` reference. Returns `null` if the underlying engine has already been freed.

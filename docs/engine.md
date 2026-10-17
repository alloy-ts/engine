# Engine API Documentation

The `Engine` class in `@alloy-ts/engine` provides a unified interface wrapping both an embedded QuickJS engine (`rquickjs::Runtime`) and the host Node-API environment (`napi::Env`).

## Overview

```ts
import { Engine, Loader } from "@alloy-ts/engine";

// Create engine (optionally passing a module loader)
const engine = new Engine();
```

`Engine` supports both camelCase (JS standard) and snake_case method names.

## Methods

### Embedded QuickJS Engine API

- `createContext()` / `create_context()`: `Context`
  Creates a new isolated QuickJS evaluation realm on this engine.

- `setLoader(loader: Loader)` / `set_loader(loader: Loader)`: `void`
  Attaches or replaces the module loader for this engine.

- `weak()`: `WeakEngine`
  Obtains a weak reference to this engine that does not keep it alive.

- `setMemoryLimit(limitInBytes: number)` / `set_memory_limit(limitInBytes: number)`: `void`
  Sets the maximum memory threshold for the QuickJS runtime.

- `setMaxStackSize(sizeInBytes: number)` / `set_max_stack_size(sizeInBytes: number)`: `void`
  Sets the maximum call stack size for the engine.

- `setGcThreshold(thresholdInBytes: number)` / `set_gc_threshold(thresholdInBytes: number)`: `void`
  Configures the garbage collection trigger threshold.

- `setInfo(info: string)` / `set_info(info: string)`: `void`
  Sets an informational label for debugging.

- `runGc()` / `run_gc()`: `void`
  Forces a garbage collection cycle in QuickJS.

- `memoryUsage()` / `memory_usage()`: `Record<string, number>`
  Returns memory usage statistics for the runtime.

- `isJobPending()` / `is_job_pending()`: `boolean`
  Returns whether any promise or microtask jobs are queued.

- `execPendingJob()` / `executePendingJob()` / `execute_pending_job()`: `boolean`
  Executes the next pending job in the runtime job queue. Returns `true` if a job ran.

### Host N-API Environment API

- `isDisposed()` / `is_disposed()`: `boolean`
  Returns whether the host environment handle has been unloaded.

- `createArray(length: number)` / `create_array(length: number)`: `any[]`
  Creates a host array of the specified length in Node's V8 realm.

- `createString(value: string)` / `create_string(value: string)`: `string`
  Allocates a host string value.

- `createInt32(value: number)` / `create_int32(value: number)`: `number`
  Allocates a host 32-bit signed integer.

- `createDouble(value: number)` / `create_double(value: number)`: `number`
  Allocates a host double float value.

- `hasGlobal(name: string)` / `has_global(name: string)`: `boolean`
  Checks if the host global object has the specified property.

- `getNapiVersion()` / `get_napi_version()`: `number`
  Gets the N-API version supported by the host.

- `makeObject()` / `make_object()`: `object`
  Builds a host JS object using the host N-API environment.

- `setInstanceData(data: string)` / `set_instance_data(data: string)`: `void`
  Stores host-level instance data associated with the addon.

- `getInstanceData()` / `get_instance_data()`: `string | null`
  Retrieves previously stored host instance data.

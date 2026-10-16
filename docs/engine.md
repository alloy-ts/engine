# Engine API

The `Engine` class provides high-level bindings combining both N-API environment context (`napi::Env`) operations and embedded QuickJS runtime (`rquickjs::Runtime`) management.

## Usage

```ts
import { Engine } from "@alloy-ts/engine";

const engine = new Engine();
```

## Methods

### N-API Environment Context Methods

- **`is_disposed(): boolean`**
  Returns whether the owning addon or N-API context has been unloaded.

- **`create_array(len: number): unknown`**
  Allocates a JavaScript Array of the requested length.

- **`create_string(value: string): unknown`**
  Creates a JavaScript String value.

- **`create_int32(int: number): unknown`**
  Creates a JavaScript Number from a 32-bit signed integer.

- **`create_double(value: number): unknown`**
  Creates a JavaScript Number from a 64-bit float.

- **`has_global(name: string): boolean`**
  Checks whether a property exists on the global object.

- **`get_napi_version(): number`**
  Returns the N-API version of the host runtime.

- **`make_object(): object`**
  Creates a new JS object using the captured N-API environment.

- **`set_instance_data(value: string): void`**
  Stores instance-specific state on the current agent.

- **`get_instance_data(): string | null`**
  Retrieves stored instance-specific state.

### QuickJS Runtime Methods

- **`is_job_pending(): boolean`**
  Checks whether at least one job (such as a pending promise) is queued in the QuickJS runtime.

- **`execute_pending_job(): boolean`**
  Executes the first pending job in the QuickJS queue. Returns `true` if a job was executed, or `false` if the queue was empty.

- **`execPendingJob(): boolean`**
  Alias for `execute_pending_job()`.

- **`exec_pending_job(): boolean`**
  Alias for `execute_pending_job()`.

- **`run_gc(): void`**
  Manually triggers the QuickJS cyclic garbage collector.

- **`set_memory_limit(limitBytes: number): void`**
  Sets a maximum memory limit (in bytes) for the QuickJS runtime. Setting `0` means unlimited memory.

- **`set_max_stack_size(limitBytes: number): void`**
  Sets a maximum call stack size limit (in bytes) for the QuickJS runtime.

- **`set_gc_threshold(thresholdBytes: number): void`**
  Sets the memory threshold (in bytes) that triggers automatic garbage collection cycles.

- **`set_file_loader(paths?: string[]): void`**
  Configures a file-based module resolver and loader searching the provided directory paths.

# Engine

The `Engine` class wraps both the host Node-API context (`napi::Env`) and the underlying QuickJS `Runtime`.

## Construction

```ts
import { Engine } from "@alloy-ts/engine";

const engine = new Engine();
```

Constructed without JavaScript arguments; Node-API injects the host environment context automatically.

## Methods

### Node-API Context Methods

- **`create_string(value: string): unknown`**: Creates a JavaScript string through the environment.
- **`create_int32(value: number): unknown`**: Creates a JavaScript 32-bit integer number.
- **`create_double(value: number): unknown`**: Creates a JavaScript double-precision float number.
- **`create_array(length: number): Array<unknown>`**: Allocates a JavaScript array of the specified length.
- **`make_object(): object`**: Constructs a new JavaScript object via the environment context.
- **`has_global(name: string): boolean`**: Checks if the global object possesses the given property name.
- **`get_napi_version(): number`**: Reports the host Node-API version.
- **`set_instance_data(value: string): void`**: Persists agent/addon instance data.
- **`get_instance_data(): string | null`**: Reads back the persisted agent instance data.
- **`is_disposed(): boolean`**: Returns `true` if the addon environment has been unloaded.

### QuickJS Runtime Methods

- **`set_info(info: string): void`**: Sets user metadata/info string on the runtime.
- **`set_memory_limit(limitBytes: number): void`**: Configures maximum memory limit in bytes (0 for unlimited).
- **`set_max_stack_size(limitBytes: number): void`**: Configures maximum stack size in bytes.
- **`set_gc_threshold(thresholdBytes: number): void`**: Configures memory threshold before triggering garbage collection.
- **`set_dump_flags(flags: number): void`**: Configures debug memory dumping flags.
- **`run_gc(): void`**: Manually triggers cyclic garbage collection.
- **`is_job_pending(): boolean`**: Returns `true` if pending async jobs exist in the runtime queue.
- **`execPendingJob(): boolean`**: Executes the first pending job in the queue.
- **`execute_pending_job(): boolean`**: Snake_case alias for `execPendingJob`.
- **`memory_usage(): JsMemoryUsage`**: Returns memory usage statistics (malloc size, object count, etc.).
- **`set_file_loader(): void`**: Registers default file module resolver and script loader.

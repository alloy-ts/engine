# Engine API (`Engine`)

`Engine` is a JavaScript class wrapping both the Node-API environment (`napi::Env`) and the QuickJS runtime (`rquickjs::Runtime`).

## Constructor

```ts
import { Engine } from "@alloy-ts/engine";

const engine = new Engine();
```

Constructs a new `Engine` instance. The Node-API environment handle is automatically injected by N-API upon construction.

## Methods

### N-API Context Operations

- `engine.is_disposed()` / `engine.isDisposed()`: Returns `boolean` indicating whether the addon context has been unloaded.
- `engine.create_array(len: number)`: Allocates a JavaScript array of length `len`.
- `engine.create_string(val: string)`: Builds a JavaScript string value.
- `engine.create_int32(int: number)`: Builds a JavaScript 32-bit integer number value.
- `engine.create_double(double: number)`: Builds a JavaScript float/double number value.
- `engine.has_global(name: string)`: Returns `boolean` checking if `globalThis` has property `name`.
- `engine.get_napi_version()`: Returns `number` representing the host Node-API version.
- `engine.make_object()`: Creates a new JavaScript object via the environment context.
- `engine.set_instance_data(data: string)`: Persists agent-specific instance state.
- `engine.get_instance_data()`: Reads back persisted instance state.

### QuickJS Runtime Operations

- `engine.run_gc()` / `engine.runGc()`: Triggers garbage collection in the QuickJS engine.
- `engine.set_memory_limit(limit: number)`: Sets the maximum memory limit in bytes (0 for unlimited).
- `engine.set_max_stack_size(limit: number)`: Sets maximum stack size limit in bytes.
- `engine.set_gc_threshold(threshold: number)`: Sets memory threshold for garbage collection cycles.
- `engine.set_dump_flags(flags: number)`: Configures debug dump flags.
- `engine.set_info(info: string)`: Sets the descriptive info string for the runtime.
- `engine.is_job_pending()` / `engine.isPendingJob()`: Returns `boolean` checking if pending async jobs exist in the queue.
- `engine.execute_pending_job()` / `engine.execPendingJob()`: Executes the first pending job in the queue. Returns `boolean` indicating if a job was executed.
- `engine.set_loader(fileResolver, scriptLoader)`: Attaches a custom file resolver and script loader.
- `engine.set_builtin_loader(builtinResolver)`: Attaches a builtin module resolver and loader.

# Engine API Documentation

The `Engine` class represents both an **embedded QuickJS JavaScript engine** (backed by `rquickjs`) and a view of the **host N-API environment** (Node's V8 `napi::Env`).

## Constructor

```ts
const engine = new Engine(loader?: Loader);
```

- `loader` *(optional)*: An instance of `Loader` containing pre-registered ES modules.

---

## Facet A: QuickJS Embedded Runtime Methods

### Context Creation
- `engine.create_context()` / `engine.createContext()`: Returns a new `Context` bound to this engine instance.

### Job & Promise Draining
- `engine.is_job_pending()` / `engine.isJobPending()`: Returns `boolean` indicating if microtask/promise jobs are queued.
- `engine.execute_pending_job()` / `engine.executePendingJob()` / `engine.execPendingJob()`: Executes one pending job from the queue. Returns `true` if a job was executed, `false` otherwise.

### Garbage Collection & Memory Management
- `engine.run_gc()`: Triggers a garbage collection cycle in QuickJS.
- `engine.memory_usage()`: Returns a statistics object detailing memory usage.
- `engine.set_memory_limit(limitInBytes: number)`: Sets maximum memory allocation limit.
- `engine.set_max_stack_size(stackSizeInBytes: number)`: Sets maximum call stack size.
- `engine.set_gc_threshold(thresholdInBytes: number)`: Sets memory threshold for automatic GC.

### Configuration & Weak References
- `engine.set_loader(loader: Loader)`: Replaces or attaches a module loader.
- `engine.set_info(infoString: string)`: Sets engine description string.
- `engine.weak()`: Returns a `WeakEngine` reference.

---

## Facet B: Host N-API Environment Methods

- `engine.create_string(value: string)`: Constructs a JS string in the host realm.
- `engine.create_int32(value: number)`: Constructs a host `i32` number.
- `engine.create_double(value: number)`: Constructs a host `f64` double.
- `engine.create_array(length: number)`: Allocates an array of given length in the host realm.
- `engine.make_object()`: Creates a plain object `{ ok: true, engine: "napi" }` in the host realm.
- `engine.has_global(name: string)`: Checks if a global variable exists on the host global object.
- `engine.get_napi_version()`: Returns the host runtime's N-API version.
- `engine.set_instance_data(data: string)`: Stores host-level state on the N-API environment.
- `engine.get_instance_data()`: Reads back persisted host-level state.
- `engine.is_disposed()`: Checks if the underlying N-API environment has been unloaded.

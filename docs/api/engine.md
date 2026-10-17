# Engine Architecture and NAPI Export Design

## Overview
The `Engine` (`JsEngine`) wraps the core JavaScript runtime along with the host NAPI environment (`napi::Env`).

## Native API Surface & NAPI Methods

### Engine Construction & Weak References
- `new Engine(loader?)`: Creates a new engine instance, optionally configured with a custom module loader.
- `engine.weak()`: Returns a `JsWeakEngine` handle that can attempt promotion back to an `Engine` via `try_ref()`.

### Configuration Methods
- `engine.set_memory_limit(limit)`: Configures maximum engine memory allocation in bytes.
- `engine.set_max_stack_size(limit)`: Configures maximum call stack size in bytes.
- `engine.set_gc_threshold(threshold)`: Configures memory threshold for automatic GC cycles.
- `engine.set_info(info)`: Sets engine informational/debugging label string.
- `engine.set_loader(loader)`: Attaches or updates the module loader/resolver.

### Runtime Control & Execution
- `engine.run_gc()`: Manually triggers cyclic garbage collection.
- `engine.memory_usage()`: Returns a memory usage statistics object.
- `engine.is_job_pending()`: Returns boolean indicating whether pending promise microtasks remain.
- `engine.execute_pending_job()`: Executes the next pending microtask job.

### Host Helpers
- `engine.is_disposed()`: Returns true if the NAPI context / addon is unloaded.
- `engine.create_array(len)` / `engine.create_string(val)` / `engine.create_int32(val)` / `engine.create_double(val)`: Constructs host JS primitives.
- `engine.has_global(name)`: Checks host global object property existence.
- `engine.get_napi_version()`: Returns N-API version of the host Node environment.
- `engine.make_object()`: Constructs host objects.
- `engine.set_instance_data(val)` / `engine.get_instance_data()`: Host state persistence helpers.

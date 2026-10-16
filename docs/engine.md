# `JsEngine` / `Engine` API Reference

`Engine` (also exported as `JsEngine`) is a JavaScript-visible class that wraps both the N-API context (`napi::Engine`) and the QuickJS runtime (`rquickjs::Runtime`). It provides safe methods to interact with the JavaScript runtime context, allocate values, query global scope, store environment-scoped instance data, configure QuickJS memory/stack limits, and manage async job queues.

## Module Import

```ts
import { Context, Engine, JsEngine } from "@alloy-ts/engine";

// `Engine` and `JsEngine` refer to the same binding class.
const engine = new JsEngine();
```

---

## Constructor

### `new JsEngine()` / `new Engine()`

Constructs a new `Engine` instance.

- **Arguments**: None (N-API injects the context handle automatically; QuickJS runtime is initialized).
- **Returns**: A new `Engine` / `JsEngine` instance.

```ts
const engine = new JsEngine();
```

---

## N-API Context API

### `is_disposed(): boolean`

Checks whether the underlying N-API context has been unloaded/disposed.

- **Returns**: `boolean` — `false` while the addon remains loaded; `true` if torn down.

```ts
if (!engine.is_disposed()) {
  console.log("Engine active");
}
```

---

### `create_array(len: number): unknown[]`

Allocates a JavaScript `Array` of the requested length in the N-API context.

```ts
const arr = engine.create_array(3);
console.log(arr.length); // 3
```

---

### `create_string(value: string): string`

Creates a JavaScript string value through the N-API context.

```ts
const str = engine.create_string("hello");
```

---

### `create_int32(int: number): number`

Builds a JavaScript number from a 32-bit signed integer.

```ts
const num = engine.create_int32(42);
```

---

### `create_double(value: number): number`

Builds a JavaScript number from a 64-bit double-precision float.

```ts
const doubleVal = engine.create_double(3.14);
```

---

### `has_global(name: string): boolean`

Queries whether a property exists on `globalThis`.

```ts
const hasMath = engine.has_global("Math"); // true
```

---

### `get_napi_version(): number`

Retrieves the N-API version supported by the host runtime.

```ts
const version = engine.get_napi_version();
```

---

### `make_object(): { ok: boolean; engine: string }`

Constructs a JavaScript object using the context with `{ ok: true, engine: "napi" }`.

```ts
const obj = engine.make_object();
```

---

### `set_instance_data(value: string): void`

Persists VM-specific state on the Agent.

```ts
engine.set_instance_data("vm-state");
```

---

### `get_instance_data(): string | null`

Reads back the VM-specific state stored on the Agent.

```ts
const state = engine.get_instance_data(); // "vm-state"
```

---

## QuickJS Runtime API

### `set_info(info: string): void`

Sets descriptive information string on the QuickJS runtime.

```ts
engine.set_info("quickjs-instance");
```

---

### `set_memory_limit(limitInBytes: number): void`

Sets a maximum limit on memory (in bytes) for the QuickJS runtime. Pass `0` for unlimited memory.

```ts
engine.set_memory_limit(1024 * 1024 * 64);
```

---

### `set_max_stack_size(limitInBytes: number): void`

Sets the maximum size of the execution stack (in bytes).

```ts
engine.set_max_stack_size(256 * 1024);
```

---

### `set_gc_threshold(thresholdInBytes: number): void`

Sets a memory threshold (in bytes) to trigger garbage collection.

```ts
engine.set_gc_threshold(64 * 1024);
```

---

### `run_gc(): void`

Triggers the QuickJS garbage collector.

```ts
engine.run_gc();
```

---

### `is_job_pending(): boolean`

Queries whether any job is pending in the QuickJS queue.

```ts
const pending = engine.is_job_pending();
```

---

### `execute_pending_job(): boolean` / `execPendingJob(): boolean`

Executes the first pending job in the QuickJS queue. `execPendingJob` is an alias for `execute_pending_job`.

```ts
engine.execute_pending_job();
// Or using the alias:
engine.execPendingJob();
```

---

## QuickJS Context Creation

Pass `Engine` instances to `Context.full(engine)` or `Context.base(engine)` to create QuickJS execution contexts:

```ts
const ctx = Context.full(engine);
const result = ctx.eval("1 + 1"); // "2"
```

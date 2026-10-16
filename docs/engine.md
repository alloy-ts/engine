# `JsEngine` / `Engine` API Reference

`Engine` (also exported as `JsEngine`) is a JavaScript-visible class that wraps the QuickJS runtime (`rquickjs::Runtime`). It serves as the primary entry point for managing JavaScript execution runtimes, configuring memory/stack limits, triggering garbage collection, and executing pending job queues.

## Module Import

```ts
import { Context, Engine, JsEngine } from "@alloy-ts/engine";

// `Engine` and `JsEngine` refer to the same binding class.
const engine = new JsEngine();
```

---

## Constructor

### `new JsEngine()` / `new Engine()`

Creates a new QuickJS engine/runtime instance.

- **Arguments**: None.
- **Returns**: A new `Engine` instance.

```ts
const engine = new JsEngine();
```

---

## Instance Methods

### `set_info(info: string): void`

Sets descriptive information string on the runtime.

```ts
engine.set_info("my-quickjs-runtime");
```

---

### `set_memory_limit(limitInBytes: number): void`

Sets a maximum limit on the amount of memory (in bytes) the runtime may allocate. Pass `0` for unlimited memory.

```ts
engine.set_memory_limit(1024 * 1024 * 64); // 64 MB
```

---

### `set_max_stack_size(limitInBytes: number): void`

Sets the maximum size of the execution stack (in bytes) used by the runtime.

```ts
engine.set_max_stack_size(256 * 1024); // 256 KB
```

---

### `set_gc_threshold(thresholdInBytes: number): void`

Sets a memory threshold (in bytes) to trigger garbage collection cycles.

```ts
engine.set_gc_threshold(64 * 1024);
```

---

### `run_gc(): void`

Manually triggers the QuickJS cyclic garbage collector.

```ts
engine.run_gc();
```

---

### `is_job_pending(): boolean`

Queries whether any async/microtask job is pending in the queue.

- **Returns**: `boolean` — `true` if jobs are pending; `false` otherwise.

```ts
if (engine.is_job_pending()) {
  engine.execute_pending_job();
}
```

---

### `execute_pending_job(): boolean` / `execPendingJob(): boolean`

Executes the first pending job in the queue. `execPendingJob` is an alias for `execute_pending_job`.

- **Returns**: `boolean` — `true` if a job was executed; `false` if the queue was empty.

```ts
const executed = engine.execute_pending_job();
```

---

## Creating Contexts

`Engine` instances are passed to `Context.full(engine)` or `Context.base(engine)` to create execution contexts:

```ts
const ctx = Context.full(engine);
const result = ctx.eval("1 + 1"); // "2"
```

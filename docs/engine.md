# Engine

The `Engine` class combines Node-API context integration (`napi::Env`) with an embedded QuickJS runtime engine (`rquickjs::Runtime`).

## Constructor

```ts
import { Engine, Loader } from "@alloy-ts/engine";

const loader = new Loader();
const engine = new Engine(loader);
```

### Parameters

- `loader?: Loader` (optional): Module loader registry to resolve `import` statements within execution contexts created by this engine.

---

## Methods

### `create_context(): Context`

Creates a new execution context (`Context`) sharing this engine's runtime.

```ts
const ctx = engine.create_context();
ctx.eval("1 + 1");
```

---

### `execPendingJob(): boolean` / `execute_pending_job(): boolean`

Executes the first pending job/microtask queued in QuickJS. Returns `true` if a job was executed, or `false` if no jobs were pending.

```ts
engine.execPendingJob();
```

---

### `is_job_pending(): boolean`

Returns `true` if at least one job is pending in the queue.

```ts
if (engine.is_job_pending()) {
  engine.execPendingJob();
}
```

---

### `run_gc(): void`

Triggers a garbage collection cycle on the embedded QuickJS engine.

```ts
engine.run_gc();
```

---

### `set_memory_limit(limitInBytes: number): void`

Sets a maximum memory limit in bytes for the runtime engine. `0` specifies unlimited.

```ts
engine.set_memory_limit(10 * 1024 * 1024); // 10 MB
```

---

### `memory_usage(): Record<string, number>`

Returns runtime memory usage statistics as a JSON object matching QuickJS `JSMemoryUsage`.

```ts
const stats = engine.memory_usage();
console.log(stats.memory_used_size);
```

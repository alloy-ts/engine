# QuickJS `Engine` API Reference

The `Engine` class wraps the underlying QuickJS runtime (`rquickjs::Runtime`). It serves as the primary entry point for managing JavaScript execution, memory usage limits, stack size, garbage collection, and module loaders.

## Constructor

```ts
const engine = new Engine();
```

Creates a new QuickJS runtime instance.

## Methods

### `setInfo(info: string): void`

Sets the informational description string for the runtime.

---

### `setMemoryLimit(limit: number): void`

Sets a limit (in bytes) on the maximum memory the runtime is permitted to allocate. Setting `limit` to `0` implies no limit.

---

### `setMaxStackSize(limit: number): void`

Sets the maximum stack size (in bytes) for JavaScript execution. Defaults to 256 KiB.

---

### `setGcThreshold(threshold: number): void`

Sets the memory threshold (in bytes) that triggers automatic cyclic garbage collection.

---

### `runGc(): void`

Manually triggers the QuickJS cyclic garbage collector.

---

### `isJobPending(): boolean`

Returns `true` if there is at least one pending async job or promise reaction in the runtime queue.

---

### `executePendingJob(): boolean`

Executes the next pending async job in the runtime queue. Returns `true` if a job was executed, or `false` if the queue was empty.

---

### `setLoader(resolver: Resolver, loader: Loader): void`

Configures custom module resolution and loading strategies for ES modules evaluated by the runtime.

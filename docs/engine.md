# Engine

The `Engine` class encapsulates both the Node-API (`napi::Env`) environment context and the underlying QuickJS `Runtime`. It provides methods for creating JavaScript primitive values, managing memory limits, running garbage collection, and managing async jobs.

## Construction

```ts
import { Engine } from "@alloy-ts/engine";

const engine = new Engine();
```

The constructor takes no JavaScript arguments; NAPI injects the host environment context automatically upon instantiation.

## N-API Context Methods

### `createString(value: string): string`

Creates a JavaScript string from a string value.

### `createInt32(value: number): number`

Creates a JavaScript number from an integer value.

### `createDouble(value: number): number`

Creates a JavaScript number from a floating point double.

### `createArray(length: number): Array<unknown>`

Allocates a JavaScript array of the requested length.

### `hasGlobal(name: string): boolean`

Checks whether a named property exists on `globalThis`.

### `getNapiVersion(): number`

Returns the N-API version supported by the host runtime.

### `makeObject(): { ok: boolean; engine: string }`

Creates a JavaScript object via the captured N-API context.

### `setInstanceData(value: string): void`

Stores VM-specific string state on the Agent.

### `getInstanceData(): string | null`

Reads back the persisted VM-specific string state.

### `isDisposed(): boolean`

Returns `true` if the owning addon context has been unloaded/disposed.

## QuickJS Runtime Methods

### `setInfo(info: string): void`

Sets the informational identifier string for the runtime.

### `setMemoryLimit(limitBytes: number): void`

Sets a limit on the maximum memory bytes the runtime can allocate (0 for unlimited).

### `setMaxStackSize(stackBytes: number): void`

Sets the maximum stack size limit for JS execution.

### `setGcThreshold(thresholdBytes: number): void`

Sets the memory allocation threshold that triggers automatic garbage collection.

### `runGc(): void`

Manually runs the QuickJS garbage collector to collect cyclic references.

### `isJobPending(): boolean`

Returns `true` if there is at least one pending asynchronous job in the job queue.

### `executePendingJob(): boolean` / `execPendingJob(): boolean`

Executes the next pending asynchronous job in the job queue. Returns `true` if a job was executed, `false` if the queue was empty.

### `setLoader(loader: Loader): void`

Configures module path resolution and builtin module sources using a `Loader` instance.

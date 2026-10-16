# Engine API

The `Engine` class wraps the Node-API environment context (`napi::Env`) and QuickJS runtime.

## Constructor

```ts
const engine = new Engine();
```

Constructs an instance of `Engine`. NAPI injects the active context handle automatically.

## N-API Context Methods

### `createArray(len: number): unknown[]`
Allocates a JavaScript array of length `len`.

### `createString(value: string): string`
Creates a JavaScript string value from the given string.

### `createInt32(value: number): number`
Creates a JavaScript 32-bit signed integer number.

### `createDouble(value: number): number`
Creates a JavaScript 64-bit floating point number.

### `hasGlobal(name: string): boolean`
Returns whether the global object contains a property named `name`.

### `getNapiVersion(): number`
Returns the Node-API version supported by the host runtime.

### `makeObject(): object`
Constructs an object via the N-API context.

### `setInstanceData(data: string): void`
Persists instance data string associated with the current agent context.

### `getInstanceData(): string | null`
Reads back the persisted instance data string.

### `isDisposed(): boolean`
Returns `true` if the addon instance or context has been disposed.

## QuickJS Runtime Methods

### `setMemoryLimit(limit: number): void`
Sets the maximum memory limit in bytes for the QuickJS runtime.

### `setMaxStackSize(limit: number): void`
Sets the maximum stack size in bytes for the QuickJS runtime.

### `setGcThreshold(threshold: number): void`
Sets the memory threshold in bytes for triggering garbage collection.

### `runGc(): void`
Triggers garbage collection in the QuickJS runtime.

### `isJobPending(): boolean`
Returns `true` if there is at least one pending job in the event loop.

### `executePendingJob(): boolean`
Executes the first pending job in the QuickJS job queue. Returns `true` if a job was executed.

### `execPendingJob(): boolean`
Alias for `executePendingJob()`.

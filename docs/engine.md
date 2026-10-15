# Engine

`Engine` is a JavaScript class exported by `@alloy-ts/engine`. It wraps both the host Node-API context handle (`napi::Env`) and the embedded QuickJS execution runtime.

## Usage

```ts
import { Engine } from "@alloy-ts/engine";

const engine = new Engine();
```

## Node-API Context Methods

### `isDisposed(): boolean`
Returns whether the underlying native context has been unloaded/disposed.

### `createArray(length: number): Array<unknown>`
Creates a new JavaScript array with the specified length using the context.

### `createString(value: string): string`
Creates a JavaScript string in the host VM.

### `createInt32(value: number): number`
Creates a 32-bit integer number value in the host VM.

### `createDouble(value: number): number`
Creates a double precision floating point number value in the host VM.

### `hasGlobal(name: string): boolean`
Checks if a given named property exists on the global object.

### `getNapiVersion(): number`
Returns the Node-API version supported by the host runtime.

### `makeObject(): object`
Constructs a new JavaScript object via the context.

### `setInstanceData(value: string): void`
Persists VM-specific state on the native Agent.

### `getInstanceData(): string | null`
Reads back the persisted VM-specific state from the native Agent.

## QuickJS Runtime Methods

### `setMemoryLimit(limitBytes: number): void`
Sets a limit on the maximum amount of memory (in bytes) that the QuickJS runtime can use.

### `setMaxStackSize(stackSizeBytes: number): void`
Sets a limit on the maximum stack size (in bytes) used by the QuickJS runtime.

### `setGcThreshold(thresholdBytes: number): void`
Sets a memory threshold for QuickJS garbage collection cycles.

### `runGc(): void`
Triggers QuickJS cyclic reference garbage collection manually.

### `setLoader(loader: Loader): void`
Registers module loader and resolver configurations on the runtime.

### `isJobPending(): boolean`
Returns `true` if there is at least one pending QuickJS job in the queue.

### `executePendingJob(): boolean`
Executes the next pending job in the QuickJS queue. Returns `true` if a job was executed, or `false` if the queue was empty.

### `execPendingJob(): boolean`
Alias for `executePendingJob()`.

### `createContext(): Context`
Creates and returns a new execution `Context` bound to this `Engine` runtime.

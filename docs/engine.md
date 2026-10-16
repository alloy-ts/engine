# Engine API

The `Engine` class is a unified NAPI binding that encapsulates both the Node-API environment handle (`napi::Env`) and QuickJS runtime execution capabilities (`rquickjs::Runtime`).

## Overview

```ts
import { Engine } from "@alloy-ts/engine";

const engine = new Engine();
```

## Methods

### Node-API Context Operations

- `createArray(len: number): unknown[]`
  Allocates a JavaScript Array with the specified length.

- `createString(value: string): string`
  Creates a JavaScript string from a primitive string.

- `createInt32(value: number): number`
  Creates a JavaScript number from an integer.

- `createDouble(value: number): number`
  Creates a JavaScript number from a floating point double.

- `hasGlobal(name: string): boolean`
  Checks if a property exists on the global object.

- `getNapiVersion(): number`
  Returns the N-API version supported by the host runtime.

- `makeObject(): object`
  Creates an object via the captured N-API context.

- `setInstanceData(value: string): void`
  Stores VM-specific state on the current agent instance.

- `getInstanceData(): string | null`
  Retrieves VM-specific state associated with the current agent instance.

- `isDisposed(): boolean`
  Returns `true` if the engine handle has been disposed due to addon unload.

### QuickJS Runtime Operations

- `setMemoryLimit(limit: number): void`
  Sets the maximum memory limit in bytes for the QuickJS runtime.

- `setMaxStackSize(limit: number): void`
  Sets the maximum stack size limit in bytes for the QuickJS runtime.

- `setGcThreshold(threshold: number): void`
  Sets the memory allocation threshold in bytes for triggering garbage collection.

- `runGc(): void`
  Manually triggers a cycle of cyclic garbage collection.

- `isJobPending(): boolean`
  Checks whether at least one job is pending in the QuickJS job queue.

- `executePendingJob(): boolean`
  Executes the next pending job in the queue. Returns `true` if a job was executed.

- `execPendingJob(): boolean`
  Alias for `executePendingJob()`.

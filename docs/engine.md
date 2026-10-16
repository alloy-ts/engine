# Engine API Documentation

The `Engine` class provides high-level bindings that combine both the N-API context (`napi::Env`) and QuickJS runtime capabilities.

## Constructor

```ts
const engine = new Engine();
```

Constructs a new `Engine` instance without JavaScript arguments.

## N-API Context Methods

- `isDisposed(): boolean`: Returns whether the engine's N-API context has been disposed.
- `createArray(len: number): unknown[]`: Creates a JS array of length `len`.
- `createString(value: string): unknown`: Creates a JS string.
- `createInt32(value: number): unknown`: Creates a JS int32 number.
- `createDouble(value: number): unknown`: Creates a JS double number.
- `hasGlobal(name: string): boolean`: Checks if the global object has the named property.
- `getNapiVersion(): number`: Returns the host N-API version.
- `makeObject(): object`: Creates a JS object with default properties.
- `setInstanceData(data: string): void`: Persists VM-specific state on the agent.
- `getInstanceData(): string | null`: Reads back persisted VM-specific state.

## QuickJS Runtime Methods

- `setInfo(info: string): void`: Sets engine information.
- `setMemoryLimit(limit: number): void`: Sets the maximum memory limit in bytes.
- `setMaxStackSize(limit: number): void`: Sets the maximum stack size.
- `setGcThreshold(threshold: number): void`: Sets the memory threshold for garbage collection.
- `setDumpFlags(flags: number): void`: Sets debug flags for dumping memory.
- `runGc(): void`: Manually runs the QuickJS garbage collector.
- `isJobPending(): boolean`: Checks if any QuickJS async job is pending execution.
- `execPendingJob(): boolean`: Executes the first pending QuickJS job.
- `executePendingJob(): boolean`: Alias for `execPendingJob()`.

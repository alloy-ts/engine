# Engine Documentation

`Engine` provides NAPI bindings to QuickJS runtime management and Node-API context operations.

## Overview

The `Engine` class combines QuickJS runtime operations (memory limits, garbage collection, pending job execution) and Node-API environment helper methods.

## API Reference

### Constructor

```ts
const engine = new Engine();
```

Creates a new `Engine` instance wrapping QuickJS `Runtime`.

### Methods

#### `set_memory_limit(limit: number): void`

Set maximum memory limit (in bytes) for the QuickJS runtime. Pass `0` for unlimited memory.

#### `set_max_stack_size(limit: number): void`

Set maximum stack size (in bytes) for the QuickJS engine.

#### `set_gc_threshold(threshold: number): void`

Set memory threshold in bytes for triggering garbage collection.

#### `run_gc(): void`

Trigger garbage collection cycle manually.

#### `is_job_pending(): boolean`

Returns `true` if there are pending jobs in the engine queue.

#### `execute_pending_job(): boolean`

Executes the first pending job in the QuickJS job queue. Returns `true` if a job was executed, `false` if queue was empty.

#### `execPendingJob(): boolean`

Alias for `execute_pending_job()`.

#### N-API Helpers

- `create_array(len: number): unknown`
- `create_string(value: string): unknown`
- `create_int32(int: number): unknown`
- `create_double(value: number): unknown`
- `has_global(name: string): boolean`
- `get_napi_version(): number`
- `make_object(): object`
- `set_instance_data(value: string): void`
- `get_instance_data(): string | null`
- `is_disposed(): boolean`

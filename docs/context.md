# Context API

The `Context` class wraps a QuickJS execution context (`rquickjs::Context`).

## Overview

```ts
import { Context, Engine } from "@alloy-ts/engine";

const engine = new Engine();
const ctx = Context.full(engine);
```

## Static Constructors

- **`Context.new(): Context`**
  Creates a standalone context with its own private runtime.

- **`Context.fromRaw(engine: Engine): Context` / `Context.from_raw(engine: Engine): Context`**
  Creates a context from a given Engine instance.

- **`Context.base(engine?: Engine): Context`**
  Creates a base context with standard minimal functions registered.

- **`Context.custom(engine?: Engine): Context`**
  Creates a context with custom intrinsics registered.

- **`Context.full(engine?: Engine): Context`**
  Creates a context with all standard intrinsics registered.

- **`Context.builder(): Context`**
  Returns a context created via context builder configuration.

## Prototype Methods

- **`asRaw(): string` / `as_raw(): string`**
  Returns the raw JSContext pointer address string.

- **`getEnginePtr(): string` / `get_engine_ptr(): string`**
  Returns the pointer address string of the associated JSRuntime/JSEngine.

- **`Engine(): Runtime` / `engine(): Runtime`**
  Returns a handle to the associated runtime engine.

- **`with(callback: () => any): any`**
  Locks the runtime and executes `callback` safely within the context lock scope.

- **`eval(source: string): any`**
  Evaluates `source` as a global script in strict mode and returns the result.

- **`eval_with_filename(source: string, filename: string): any`**
  Evaluates `source` with a specified filename for error reporting.

- **`compile(source: string, filename: string, strict?: boolean): void`**
  Validates syntax / compiles code without executing it.

- **`eval_module(source: string): any`**
  Evaluates `source` as an ES module (`import` / `export` allowed) and returns an object with all module exports.

- **`json_parse(json: string): any`**
  Parses a JSON string inside QuickJS and returns the value.

- **`global_keys(): string[]`**
  Returns property names on the global object (`Object.getOwnPropertyNames(globalThis)`).

- **`run_gc(): void`**
  Triggers garbage collection.

- **`execute_pending_job(): boolean`**
  Runs one pending job microtask. Returns `true` if a job ran.

- **`has_exception(): boolean`**
  Returns `true` if a JS exception is currently pending.

- **`catch(): any`**
  Catches and clears the pending JS exception, returning it as a JS value.

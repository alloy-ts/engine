# Context API

The `Context` class wraps a QuickJS execution context (`rquickjs::Context`).

## Overview

```ts
import { Context } from "@alloy-ts/engine";

// Standalone context with its own private runtime
const ctx = Context.new();
```

Or created from an `Engine` or `Runtime`:

```ts
import { Engine } from "@alloy-ts/engine";

const engine = new Engine();
const ctx = engine.create_context();
```

## Methods

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

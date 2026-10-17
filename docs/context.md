# Context API Documentation

The `Context` class represents an isolated JavaScript realm inside the embedded QuickJS engine.

## Overview

```ts
import { Context, Engine } from "@alloy-ts/engine";

// Standalone context (creates its own private engine):
const standalone = new Context();

// Shared context (shares an existing Engine):
const engine = new Engine();
const ctx = engine.createContext();
```

## Methods

- `eval(code: string)`: `any`
  Evaluates JavaScript source text as a script and returns the result marshalled to a host JSON-compatible value.

- `evalModule(code: string)` / `eval_module(code: string)`: `any`
  Evaluates source code as an ES module and returns its exported namespace object.

- `jsonParse(jsonText: string)` / `json_parse(jsonText: string)`: `any`
  Parses a JSON string using QuickJS' native JSON parser.

- `globalKeys()` / `global_keys()`: `string[]`
  Returns the enumerable property keys of the global object.

- `runGc()` / `run_gc()`: `void`
  Triggers a garbage collection cycle on the context's engine.

- `executePendingJob()` / `execute_pending_job()`: `boolean`
  Runs a pending microtask/job in this context. Returns `true` if a job executed.

- `hasException()` / `has_exception()`: `boolean`
  Returns whether an unhandled JS exception is pending in the context.

- `catch()`: `any`
  Catches and returns any pending exception marshalled into a JS value.

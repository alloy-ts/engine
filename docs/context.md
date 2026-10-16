# Context Documentation

The `Context` class represents an isolated JavaScript realm (global scope) inside the QuickJS engine.

## Overview

A `Context` provides an execution environment with its own global object (`globalThis`). Code evaluated inside a `Context` runs in QuickJS strict mode.

Contexts can be created in two ways:

1. **Engine-Attached** (Recommended): Created via `engine.createContext()` (or `engine.create_context()`). Shares the underlying runtime, settings, module loader, and memory pool of the parent `Engine`.
2. **Standalone**: Created via `new Context()`. Automatically spins up its own internal private engine runtime.

---

## Constructor

### `new Context()`

Constructs a standalone `Context` with its own private QuickJS runtime.

---

## Methods

### `eval(source: string): unknown`

Evaluates `source` as a global JavaScript script and returns the result marshalled into a host value.

- **Parameters**: `source` — JavaScript source code string.
- **Returns**: The evaluation result converted into a native JS value via JSON serialization.
- **Throws**: Throws a JS error if evaluation produces a syntax or execution exception.

### `eval_module(source: string): unknown`

Evaluates `source` as an ES module (`import` and `export` statements are allowed).

- **Parameters**: `source` — ES module source code string.
- **Returns**: The module namespace object serialized to JSON.
- **Note**: Exported functions collapse to `null` during JSON serialization; primitive values, objects, and arrays are preserved.

### `json_parse(jsonText: string): unknown`

Parses a JSON string using QuickJS' internal JSON parser and returns the result as a host value.

### `global_keys(): string[]`

Returns an array of key names defined on the context's global object (`Object.keys(globalThis)`).

### `run_gc(): void`

Triggers garbage collection for the context.

### `execute_pending_job(): boolean`

Runs a single pending microtask job within this context. Returns `true` if a job ran, `false` otherwise.

### `has_exception(): boolean`

Returns `true` if an unhandled JS exception is currently pending in this context.

### `catch(): unknown`

Catches and clears the current pending JS exception in this context, returning the exception value.

# Context

The `Context` class represents an isolated QuickJS realm (execution context) with its own `globalThis` scope.

## Overview

A `Context` can be created in two ways:

1. **Engine-bound (`engine.create_context()`)**: Shares an existing `Engine` runtime, memory limits, and module loaders with other contexts.
2. **Standalone (`new Context()`)**: Creates a context backed by its own private QuickJS engine instance.

Results returned by `eval` and `eval_module` are automatically serialized from QuickJS into host JavaScript values (via JSON marshalling).

## Constructor & Factory

```ts
import { Context, Engine } from "@alloy-ts/engine";

// Standalone context (creates a private engine)
const ctx = new Context();

// Context tied to a shared Engine instance
const engine = new Engine();
const sharedCtx = engine.create_context();
```

## API Reference

### `ctx.eval(source: string): unknown`

Evaluates `source` JavaScript code as a global script in QuickJS strict mode.
Returns the evaluation result marshalled as a host JavaScript value. Throws a JS error if evaluation fails.

```ts
const result = ctx.eval("1 + 2 * 3"); // 7
```

### `ctx.eval_module(source: string): unknown`

Evaluates `source` code as an ES Module (allowing `import` and `export` statements).
Returns the module namespace object serialized to a host value.

```ts
const ns = ctx.eval_module("export const a = 10; export const b = 20;");
// { a: 10, b: 20 }
```

### `ctx.json_parse(jsonString: string): unknown`

Parses a JSON string inside QuickJS and returns the resulting object.

```ts
const obj = ctx.json_parse('{"hello": "world"}'); // { hello: "world" }
```

### `ctx.global_keys(): string[]`

Returns an array of enumerable own property names on the context's global object (`Object.keys(globalThis)`).

```ts
const keys = ctx.global_keys();
```

### `ctx.run_gc(): void`

Triggers a garbage collection sweep for this context's underlying engine.

### `ctx.execute_pending_job(): boolean`

Runs one pending microtask/job inside this context. Returns `true` if a job ran, or `false` if no job was pending.

### `ctx.has_exception(): boolean`

Checks whether an unhandled JS exception is currently pending in this context.

### `ctx.catch(): unknown`

Catches and returns any pending exception marshalled as a host value, clearing the exception state.

## Context Scope Isolation

Each `Context` maintains its own isolated `globalThis` state:

```ts
const engine = new Engine();
const realm1 = engine.create_context();
const realm2 = engine.create_context();

realm1.eval("globalThis.value = 42;");
console.log(realm1.eval("globalThis.value")); // 42
console.log(realm2.eval("typeof globalThis.value")); // "undefined"
```

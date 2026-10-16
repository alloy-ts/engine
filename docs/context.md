# Context (`docs/context.md`)

The `Context` class represents an isolated JavaScript execution realm with its own global variables, standard intrinsics, and stack in `@alloy-ts/engine`. It wraps `rquickjs::Context`.

## Overview

```ts
import { Context, Engine } from "@alloy-ts/engine";

// Create standalone (backed by a private Engine runtime)
const ctx = new Context();

// Or create from an existing Engine instance
const engine = new Engine();
const sharedCtx = engine.create_context();
```

## Constructor

### `new Context()`

Creates a new standalone `Context` backed by a private embedded QuickJS runtime.

---

## Methods

#### `context.eval(source: string): unknown`

Evaluates `source` as a global script in QuickJS strict mode and returns the resulting value marshalled to JSON. Throws a JavaScript exception if evaluation fails.

```ts
const val = ctx.eval("1 + 2 * 3"); // 7
```

#### `context.eval_module(source: string): unknown`

Evaluates `source` as an ES module (`import` and `export` statements enabled) and returns its module namespace object as a JavaScript object.

```ts
const ns = ctx.eval_module("export const foo = 'bar';"); // { foo: "bar" }
```

#### `context.json_parse(json: string): unknown`

Parses JSON text inside QuickJS and returns the result as a JavaScript value.

```ts
const data = ctx.json_parse('{"x": 10}'); // { x: 10 }
```

#### `context.global_keys(): string[]`

Returns an array of own enumerable keys on `globalThis` (`Object.keys(globalThis)`).

#### `context.run_gc(): void`

Runs a garbage collection cycle in this context's runtime.

#### `context.execute_pending_job(): boolean`

Executes one pending microtask/job in this context. Returns `true` if a job ran.

#### `context.has_exception(): boolean`

Returns `true` if a JavaScript exception is currently pending in this context.

#### `context.catch(): unknown`

Returns and clears the currently pending JavaScript exception as a value (or `null` if none).

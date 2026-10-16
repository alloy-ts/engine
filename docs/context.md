# Context

A single execution context with its own global variables and stack, backed by QuickJS. Multiple contexts can share objects within the same `Engine` runtime.

## Creation

Contexts are created using static factory methods on `Context`:

```ts
import { Engine, Context } from "@alloy-ts/engine";

const engine = new Engine();

// Create a context with all standard QuickJS intrinsics registered
const ctx = Context.full(engine);

// Or create a base context with minimal intrinsics registered
const baseCtx = Context.base(engine);
```

## Methods

### `eval(code: string): string`

Evaluates a JavaScript code string inside the context and returns the string representation of the result.

```ts
const result = ctx.eval("1 + 1"); // "2"
```

### `evalFile(path: string): string`

Reads and evaluates a JavaScript file from disk.

### `jsonParse(json: string): string`

Parses a JSON string into a QuickJS JavaScript value and returns the stringified result.

### `jsonStringify(valueCode: string): string`

Evaluates `valueCode` and serializes the resulting JavaScript value into a JSON string.

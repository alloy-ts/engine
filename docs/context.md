# Context

`Context` represents an isolated execution context within QuickJS, complete with its own global variables and stack.

## Obtaining a Context

A `Context` is instantiated through an `Engine` runtime instance:

```ts
import { Engine } from "@alloy-ts/engine";

const engine = new Engine();
const ctx = engine.createContext();
```

## Methods

### `eval(script: string): string`

Evaluates a JavaScript code string inside the context and returns the string representation or JSON serialization of the evaluated result.

```ts
const result = ctx.eval("1 + 2");
console.log(result); // "3"
```

If evaluation fails or throws an unhandled exception in QuickJS, `eval` throws a JavaScript error describing the failure.

# Context API (`Context`)

`Context` is a JavaScript class wrapping `rquickjs::Context`. It represents an isolated execution context with its own global variables and stack within QuickJS.

## Constructors & Factories

```ts
import { Context } from "@alloy-ts/engine";

// Standard constructor
const ctx = new Context();

// Base context factory (minimal intrinsics)
const baseCtx = Context.base();

// Full context factory (all standard intrinsics)
const fullCtx = Context.full();
```

## Methods

- `ctx.eval(script: string): string`: Evaluates JavaScript `script` string in global context and returns the stringified result.
- `ctx.eval_int(script: string): number`: Evaluates JavaScript `script` and returns an integer result.
- `ctx.eval_bool(script: string): boolean`: Evaluates JavaScript `script` and returns a boolean result.

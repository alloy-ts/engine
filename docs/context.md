# JSContext API

`JSContext` represents an execution context in QuickJS with its own global variables and evaluation stack, created from an `Engine` instance.

## Usage

```ts
import { Engine, JSContext } from "@alloy-ts/engine";

const engine = new Engine();
const ctx = JSContext.createFull(engine);

const result = ctx.eval("1 + 2"); // "3"
```

## Static Methods

- **`JSContext.createFull(engine: Engine): JSContext`**
  Creates a context with all standard intrinsics registered (such as `Date`, `Promise`, `Math`, etc.).

- **`JSContext.createBase(engine: Engine): JSContext`**
  Creates a minimal base context with only essential intrinsics registered.

## Instance Methods

- **`eval(code: string): string`**
  Evaluates a string of JavaScript code inside the context and returns the result stringified as JSON.

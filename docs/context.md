# Context API Reference

`Context` represents an isolated JavaScript execution context in QuickJS, with its own global object, variables, and stack.

## Static Methods

### `Context.base(engine: Engine): Context`
Creates a minimal context with only base/required intrinsic objects registered.

### `Context.full(engine: Engine): Context`
Creates an execution context with all standard QuickJS intrinsics registered (Eval, Promise, Math, JSON, etc.).

## Instance Methods

### `eval(source: string): string`
Evaluates a string of JavaScript code within the context and returns the stringified result or throws on execution error.

## Example

```ts
import { Engine, Context } from "@alloy-ts/engine";

const engine = new Engine();
const ctx = Context.full(engine);

const result = ctx.eval("1 + 2");
console.log(result); // "3"
```

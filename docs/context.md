# Context API

The `Context` class provides execution contexts for the QuickJS engine.

## Static Factories

### `Context.base(): Context`
Creates a minimal execution context with base intrinsics registered.

### `Context.full(): Context`
Creates a full execution context with all standard QuickJS intrinsics registered.

## Methods

### `eval(code: string): string`
Evaluates the provided JavaScript `code` within the context and returns the stringified evaluation result.

```ts
import { Context } from "@alloy-ts/engine";

const ctx = Context.full();
const result = ctx.eval("1 + 2 * 3"); // "7"
```

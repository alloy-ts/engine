# Context API

The `Context` class provides an isolated JavaScript execution context powered by QuickJS.

## Usage

```ts
import { Context } from "@alloy-ts/engine";

// Create a full execution context
const ctx = Context.full();

// Evaluate code in context
const result = ctx.eval("1 + 1"); // "2"
```

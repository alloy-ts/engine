---
title: 'Context'
description: Isolated QuickJS JavaScript execution context.
---

# Context

The `Context` class provides an isolated JavaScript execution context with its own global variables and execution stack.

## Usage

```ts
import { Context } from "@alloy-ts/engine";

const ctx = new Context();

// Evaluate JavaScript expression
const jsonResult = ctx.eval("1 + 2"); // "3"

// Trigger garbage collection for cyclic references
ctx.run_gc();
```

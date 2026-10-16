# Context

`Context` provides JavaScript execution context management powered by QuickJS.

## Overview

A `Context` represents a single execution context with its own global variables and stack.

## Usage

```ts
import { Context } from "@alloy-ts/engine";

const ctx = new Context();
const result = ctx.eval("1 + 1"); // "2"
```

## Methods

- **`constructor()`**: Creates a new QuickJS execution context.
- **`eval(code: string): string`**: Evaluates a JavaScript string within the context and returns the result stringified.

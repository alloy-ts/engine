# QuickJS Context API

The `QuickJSContext` class exposes safe high-level bindings to a QuickJS execution context (`rquickjs::Context`).

## Overview

A `QuickJSContext` represents a single JavaScript execution environment with its own global variables and stack.

```ts
import { QuickJSContext } from "@alloy-ts/engine";

const ctx = new QuickJSContext();
```

## Methods

- `constructor()`
  Creates a new QuickJS execution context initialized with standard intrinsics.

- `eval(code: string): string`
  Evaluates a string of JavaScript code inside the context and returns the string representation of the result.

  ```ts
  const result = ctx.eval("1 + 2");
  console.log(result); // "3"
  ```

- `eval_json(code: string): string`
  Evaluates a string of JavaScript code inside the context and returns the JSON-stringified representation of the result.

  ```ts
  const jsonResult = ctx.eval_json("({ a: 1, b: 2 })");
  console.log(jsonResult); // "{\"a\":1,\"b\":2}"
  ```

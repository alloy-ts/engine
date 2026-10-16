# Context Documentation

`Context` provides NAPI bindings to QuickJS execution contexts (`rquickjs::Context`).

## Overview

A `Context` represents an isolated JavaScript execution context with its own global object and scope, managed within a QuickJS runtime.

## API Reference

### Constructor

```ts
import { Context } from "@alloy-ts/engine";

const ctx = new Context();
```

Creates a new execution context with standard intrinsics registered.

### Methods

#### `eval(script: string): string`

Evaluates a string of JavaScript code inside the context and returns the JSON-stringified result.

```ts
const result = ctx.eval("1 + 2");
console.log(result); // "3"
```

Throws an error if the script contains a syntax error or runtime exception.

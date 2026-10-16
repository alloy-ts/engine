---
title: 'Compiler'
description: oxc compiler and transformer bindings for JavaScript and TypeScript.
---

# Compiler

The `Compiler` class wraps the oxc compiler and parser, executing transformations on source text and serializing ESTree AST outputs.

## Usage

```ts
import { Compiler } from "@alloy-ts/engine";

const compiler = new Compiler();

// Execute compilation
const estreeJson = compiler.execute("const x: number = 42;", "ts", "src/app.ts");
```

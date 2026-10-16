# Compiler

`Compiler` provides high-performance JavaScript/TypeScript parsing and AST serialization backed by [OXC](https://oxc.rs).

## Overview

The `Compiler` class compiles or transforms source code into an ESTree-compatible JSON representation.

## Usage

```ts
import { Compiler } from "@alloy-ts/engine";

const compiler = new Compiler();
const astJson = compiler.execute("const x: number = 42;", "ts", "src/index.ts");
const ast = JSON.parse(astJson);
console.log(ast.type); // "Program"
```

## Methods

- **`constructor()`**: Instantiates a new OXC compiler wrapper.
- **`execute(sourceText: string, sourceType?: string, sourcePath?: string): string`**:
  Parses `sourceText` using OXC parser and returns the serialized ESTree AST JSON string.
  - `sourceText`: Source code string to parse.
  - `sourceType`: Optional override for language/file extension (e.g. `"ts"`, `"tsx"`, `"jsx"`, `"js"`).
  - `sourcePath`: Optional file path used for language inference and error reporting.
  - Throws an Error if syntax or parse errors occur.

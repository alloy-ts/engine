# Compiler API

The `Compiler` class provides NAPI bindings to oxc's AST transformation and compilation pipeline.

## Overview

```ts
import { Compiler } from "@alloy-ts/engine";

const compiler = new Compiler();
```

## Methods

- `constructor()`
  Creates a new `Compiler` instance.

- `execute(sourceText: string, sourceType?: string, sourcePath?: string): string`
  Executes oxc compilation on the provided `sourceText`. Parses the code using the given `sourceType` (or infers it from `sourcePath`) and returns an ESTree JSON string representation of the AST.

  Throws an exception with rendered diagnostics if compilation encounters syntax or fatal parse errors.

  ```ts
  const compiler = new Compiler();
  const astJson = compiler.execute("const x = 42;", "ts", "src/index.ts");
  const ast = JSON.parse(astJson);
  console.log(ast.type); // "Program"
  ```

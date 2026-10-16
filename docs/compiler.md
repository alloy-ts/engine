# Compiler API

The `Compiler` class provides a high-level NAPI interface to OXC compilation and ESTree serialization.

## Overview

```ts
import { Compiler } from "@alloy-ts/engine";

const compiler = new Compiler();
const estreeJson = compiler.execute(
  "const x = 1;",
  "js", // optional source_type ("js", "ts", "jsx", "tsx")
  "index.js", // optional source_path
);
```

## Methods

- **`execute(sourceText: string, sourceType?: string, sourcePath?: string): string`**
  Parses `sourceText` using OXC parser and serializes the resulting AST into compact ESTree JSON. Throws a JS exception with diagnostic error messages if syntax errors are present.

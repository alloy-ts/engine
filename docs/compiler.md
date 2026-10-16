# Compiler (`docs/compiler.md`)

The `Compiler` class provides AST parsing and serialization using `oxc_parser` and `oxc_estree` in `@alloy-ts/engine`.

## Overview

```ts
import { Compiler } from "@alloy-ts/engine";

const compiler = new Compiler();
const estreeJson = compiler.execute("const a = 1 + 2;", "ts", "app.ts");
```

## Constructor

### `new Compiler()`

Creates a new `Compiler` instance.

---

## Methods

#### `compiler.execute(sourceText: string, sourceType?: string, sourcePath?: string): string`

Parses `sourceText` into an ESTree AST and serializes it to a compact JSON string.

- **`sourceText`**: The JavaScript / TypeScript source code to parse.
- **`sourceType`** _(optional)_: Language or file extension override (e.g. `"ts"`, `"tsx"`, `"js"`, `"jsx"`).
- **`sourcePath`** _(optional)_: File path used for source type inference if `sourceType` is omitted. Defaults to `"index.js"`.

Throws an error containing diagnostic error messages if syntax or parsing errors occur.

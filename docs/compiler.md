# Compiler

NAPI wrapper around the Oxc compiler/parser. Parses JavaScript/TypeScript source code and serializes the resulting AST into compact ESTree JSON.

## Usage

```ts
import { Compiler } from "@alloy-ts/engine";

const compiler = new Compiler();

// Compile/parse source text into ESTree JSON
const astJson = compiler.execute("const x: number = 42;", undefined, "src/app.ts");
const ast = JSON.parse(astJson);
```

## Methods

### `execute(sourceText: string, sourceType?: string, sourcePath?: string): string`

Executes compilation/parsing on `sourceText`.

- `sourceText`: The source code string to parse.
- `sourceType`: Optional override for source language or extension (`"ts"`, `"tsx"`, `"jsx"`, etc.).
- `sourcePath`: Optional path used for source type inference (defaults to `"index.js"` if omitted).

Returns compact ESTree JSON string on success, or throws an error with diagnostics on syntax errors.

# `Compiler` API Reference

`Compiler` is a JavaScript-visible class that wraps oxc's parser and ESTree compact serializer. It compiles JavaScript and TypeScript source text directly into ESTree-compatible compact JSON.

## Module Import

```ts
import { Compiler } from "@alloy-ts/engine";

const compiler = new Compiler();
```

---

## Constructor

### `new Compiler()`

Creates a new `Compiler` instance.

- **Arguments**: None.
- **Returns**: A new `Compiler` instance.

```ts
const compiler = new Compiler();
```

---

## Instance Methods

### `execute(sourceText: string, sourceType?: string, sourcePath?: string): string`

Parses and compiles `sourceText` into an ESTree compact JSON string using `oxc_parser` and `oxc_estree::CompactSerializer`.

- **Parameters**:
  - `sourceText` (`string`): The JavaScript or TypeScript source code to compile.
  - `sourceType` (`string | undefined`): Optional language / module extension override (e.g., `"js"`, `"ts"`, `"tsx"`, `"jsx"`).
  - `sourcePath` (`string | undefined`): Optional file path used to infer language and module kind (defaults to `"index.js"`).
- **Returns**: `string` — Compact JSON string representation of the ESTree `Program` AST.
- **Throws**: `Error` — Throws an error containing diagnostic messages if parsing fails or panics.

```ts
const compiler = new Compiler();

// Basic JavaScript compilation
const jsonOutput = compiler.execute("const x = 1;", undefined, "app.js");
const ast = JSON.parse(jsonOutput);
console.log(ast.type); // "Program"

// TypeScript compilation with explicit source type
const tsAst = compiler.execute("const num: number = 42;", "ts", "app.ts");
```

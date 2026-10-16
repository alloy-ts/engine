# Compiler Documentation

The `Compiler` class provides NAPI bindings to the [oxc](https://github.com/oxc-project/oxc) parser/compiler infrastructure, serializing JavaScript and TypeScript ASTs into ESTree-compatible JSON strings.

## Overview

`Compiler` allows host environments to parse JavaScript, JSX, TypeScript, and TSX files directly via native code, returning compact ESTree JSON strings.

---

## Constructor

### `new Compiler()`

Constructs a new `Compiler` instance.

```ts
import { Compiler } from "@alloy-ts/engine";

const compiler = new Compiler();
```

---

## Methods

### `execute(sourceText: string, sourceType?: string, sourcePath?: string): string`

Parses `sourceText` and returns the serialized ESTree AST JSON string.

- **Parameters**:
  - `sourceText` (`string`): The JavaScript or TypeScript source code to parse/transform.
  - `sourceType` (`string | undefined`): Optional language/extension override (e.g., `"js"`, `"ts"`, `"tsx"`, `"jsx"`).
  - `sourcePath` (`string | undefined`): Optional filename path used for language inference and error reporting (defaults to `"index.js"`).

- **Returns**: `string` — ESTree AST JSON representation.

- **Throws**: Throws an error containing diagnostic messages if parsing fails or panics due to syntax errors.

---

## Example

```ts
import { Compiler } from "@alloy-ts/engine";

const compiler = new Compiler();
const astJson = compiler.execute("const greeting: string = 'Hello, world!';", "ts", "app.ts");
const ast = JSON.parse(astJson);

console.log(ast.type); // "Program"
```

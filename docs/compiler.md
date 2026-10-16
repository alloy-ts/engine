# Compiler

The `Compiler` class is a NAPI wrapper around the [`oxc_parser`](https://github.com/oxc-project/oxc) JavaScript and TypeScript parser. It parses source text into an ESTree-compatible JSON AST string.

## Import

```ts
import { Compiler } from "@alloy-ts/engine";
```

## Usage

### Constructor

```ts
const compiler = new Compiler();
```

Creates a new `Compiler` instance.

### `execute(sourceText, sourceType?, sourcePath?)`

Parses source code text and returns the ESTree AST serialized as a JSON string.

```ts
const astJson = compiler.execute(sourceText, sourceType, sourcePath);
```

#### Parameters

- **`sourceText`** (`string`, required): The JavaScript or TypeScript source code to parse.
- **`sourceType`** (`string | undefined`, optional): Language or module type override (e.g., `"module"`, `"script"`).
- **`sourcePath`** (`string | undefined`, optional): Source file path used for language inference if `sourceType` is omitted (defaults to `"index.js"`).

#### Return Value

- **`string`**: Compact JSON string representing the parsed ESTree AST (`Program`).

#### Errors

Throws a JS `Error` containing diagnostic messages if syntax parsing fails.

## Examples

### Parsing JavaScript

```ts
import { Compiler } from "@alloy-ts/engine";

const compiler = new Compiler();
const astJson = compiler.execute("const x = 42;");
const ast = JSON.parse(astJson);

console.log(ast.type); // "Program"
```

### Parsing TypeScript with File Path

```ts
import { Compiler } from "@alloy-ts/engine";

const compiler = new Compiler();
const code = "const add = (a: number, b: number): number => a + b;";
const astJson = compiler.execute(code, undefined, "math.ts");
const ast = JSON.parse(astJson);

console.log(ast.body[0].type); // "VariableDeclaration"
```

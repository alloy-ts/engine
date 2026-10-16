# Compiler API Reference

The `Compiler` class is an oxc-backed parser/compiler exposed as a Node-API binding. It parses JavaScript or TypeScript code into an ESTree-compatible JSON string representation of the AST.

## Usage

```ts
import { Compiler } from "@alloy-ts/engine";

const compiler = new Compiler();

// Basic execution
const jsonAst = compiler.execute("const x = 42;");
const program = JSON.parse(jsonAst);

// With custom source type and file path
const tsAstJson = compiler.execute("const a: number = 10;", "ts", "src/index.ts");
```

## API

### Constructor

#### `new Compiler()`

Creates a new `Compiler` instance.

### Methods

#### `compiler.execute(sourceText, sourceType?, sourcePath?)`

Executes compilation and parsing over `sourceText` and returns an ESTree-compatible JSON string.

##### Parameters

- **`sourceText`** (`string`): The JavaScript or TypeScript source code to parse/compile.
- **`sourceType`** (`string | null | undefined`, optional): Language override or extension (e.g. `"js"`, `"jsx"`, `"ts"`, `"tsx"`). Inferred from `sourcePath` if omitted.
- **`sourcePath`** (`string | null | undefined`, optional): File path used for language inference and diagnostic reporting. Defaults to `"index.js"`.

##### Returns

- `string`: JSON string representing the parsed ESTree `Program`.

##### Throws

- Throws an `Error` containing diagnostic error messages if the source text contains syntax or parsing errors.

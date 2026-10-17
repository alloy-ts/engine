# Compiler

`Compiler` is a JavaScript class exported by `@alloy-ts/engine`. It wraps oxc's parser and compact ESTree serializer to parse source code into a compact ESTree JSON representation.

## Usage

```ts
import { Compiler } from "@alloy-ts/engine";

const compiler = new Compiler();
const jsonResult = compiler.execute("const x = 1;", "ts", "app.ts");
```

## Constructor

### `new Compiler()`

Constructs a new `Compiler` instance.

## Methods

### `execute(sourceText: string, sourceType?: string, sourcePath?: string): string`

Parses `sourceText` and returns a serialized compact ESTree JSON string representing the AST (`Program`).

#### Parameters:

- `sourceText` (`string`): The JavaScript / TypeScript source code to compile/parse.
- `sourceType` (`string | undefined`): Optional language / source-type hint (e.g. `"ts"`, `"tsx"`, `"jsx"`).
- `sourcePath` (`string | undefined`): Optional file path used for source-type inference when `sourceType` is omitted (defaults to `"index.js"`).

#### Throws:

Throws a JavaScript Error containing formatted error diagnostics if syntax errors are encountered during parsing.

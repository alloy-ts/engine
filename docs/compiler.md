# Compiler

The `Compiler` class provides a NAPI wrapper around oxc compilation/parsing into ESTree JSON strings.

## Usage

```ts
import { Compiler } from "@alloy-ts/engine";

const compiler = new Compiler();

// Execute compilation/parsing
const estreeJson = compiler.execute("const x: number = 42;", "ts", "src/app.ts");
const ast = JSON.parse(estreeJson);
```

## API

### `new Compiler()`

Creates a new `Compiler` instance.

### `execute(sourceText: string, sourceType?: string, sourcePath?: string): string`

Compiles/parses `sourceText` using `oxc_parser` and returns serialized ESTree-compatible JSON.

- `sourceText`: The JavaScript or TypeScript source code string.
- `sourceType` (optional): Override language or extension (e.g. `"ts"`, `"tsx"`).
- `sourcePath` (optional): Path used to infer source type if `sourceType` is omitted. Defaults to `"index.js"`.

Throws an error if syntax or parsing errors occur.

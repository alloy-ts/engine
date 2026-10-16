# Compiler NAPI Bindings

The `Compiler` class provides Native (Rust) oxc-backed compilation and transformation capabilities.

## Usage

```ts
import { Compiler } from "@alloy-ts/engine";

const compiler = new Compiler();

// Execute compilation on JavaScript/TypeScript source text
const result = compiler.execute(
  "const x: number = 42;",
  "ts", // Optional source type ("js", "jsx", "ts", "tsx")
  "src/app.ts", // Optional file path used for extension/module-kind inference
);
```

## API

### `new Compiler()`

Constructs a new `Compiler` instance.

### `execute(sourceText: string, sourceType?: string, sourcePath?: string): string`

Executes compilation over `sourceText`.

- `sourceText`: The source code string to compile.
- `sourceType` _(optional)_: Explicit language type string (`"js"`, `"ts"`, `"jsx"`, `"tsx"`).
- `sourcePath` _(optional)_: File path used to infer language and module kind when `sourceType` is omitted. Defaults to `"index.js"`.

Returns serialized ESTree JSON output or throws an error if syntax errors are encountered.

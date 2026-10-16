# Compiler NAPI Bindings

The `Compiler` struct provides NAPI bindings to the Oxc parser and ESTree AST generator.

## Usage

```ts
import { Compiler } from "@alloy-ts/engine";

const compiler = new Compiler();
const jsonAst = compiler.execute("const x = 42;", "js", "index.js");
console.log(JSON.parse(jsonAst));
```

## API

### `new Compiler()`

Constructs a new `Compiler` instance.

### `execute(sourceText: string, sourceType?: string, sourcePath?: string): string`

Executes parsing on `sourceText` and returns the serialized ESTree JSON AST as a string.

- `sourceText`: JavaScript or TypeScript code to parse.
- `sourceType`: Optional language type or bare extension (`"js"`, `"ts"`, `"tsx"`, `"jsx"`).
- `sourcePath`: Optional file path used for source type inference.

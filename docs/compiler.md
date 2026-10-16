# Compiler

The `Compiler` class is a NAPI wrapper around [oxc_parser](https://crates.io/crates/oxc_parser) and [oxc_estree](https://crates.io/crates/oxc_estree) that parses JavaScript / TypeScript / JSX source code and serializes the AST into ESTree-compatible JSON.

## Constructor

```ts
import { Compiler } from "@alloy-ts/engine";

const compiler = new Compiler();
```

## API Reference

### `compiler.execute(sourceText: string, sourceType?: string, sourcePath?: string): string`

Parses `sourceText` and returns the serialized ESTree JSON string.

- `sourceText`: The source code string to parse.
- `sourceType` _(optional)_: File extension or source type override (e.g., `"js"`, `"ts"`, `"jsx"`, `"tsx"`).
- `sourcePath` _(optional)_: File path used for source type inference and diagnostics (defaults to `"index.js"`).

Throws an Error if the source code contains syntax or parsing errors.

## Example Usage

```ts
import { Compiler } from "@alloy-ts/engine";

const compiler = new Compiler();

// Parse plain JavaScript
const json = compiler.execute("const x = 42;");
const program = JSON.parse(json);
console.log(program.type); // "Program"

// Parse TypeScript with custom source type and path
const tsJson = compiler.execute("const a: number = 100;", "ts", "src/index.ts");
```

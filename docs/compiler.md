# Compiler API Documentation

The `Compiler` class provides high-level NAPI bindings for code transformation and AST compilation using the `oxc_parser` pipeline.

## API Reference

### `new Compiler()`

Constructs a new `Compiler` instance.

```ts
import { Compiler } from "@alloy-ts/engine";

const compiler = new Compiler();
```

### `execute(sourceText: string, sourceType?: string, sourcePath?: string): string`

Executes parsing and AST serialization over `sourceText`.

- **`sourceText`**: The JavaScript or TypeScript code to process.
- **`sourceType`**: Optional language or module extension (`"js"`, `"jsx"`, `"ts"`, `"tsx"`).
- **`sourcePath`**: Optional file path used to infer the source type if omitted (defaults to `"index.js"`).

Returns an ESTree-compatible JSON string representation of the parsed AST.

#### Throws

Throws an error containing the accumulated diagnostic messages if syntax or parsing errors occur.

#### Example Usage

```ts
import { Compiler } from "@alloy-ts/engine";

const compiler = new Compiler();

// Parse JavaScript
const jsAstJson = compiler.execute("const a = 10;", "js", "main.js");
console.log(JSON.parse(jsAstJson));

// Parse TypeScript
const tsAstJson = compiler.execute("const x: number = 42;", "ts", "main.ts");
console.log(JSON.parse(tsAstJson));
```

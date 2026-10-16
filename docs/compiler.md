# Compiler

The `Compiler` class provides a Node-API wrapper around the OXC JavaScript/TypeScript parser and ESTree serializer.

## Construction

```ts
import { Compiler } from "@alloy-ts/engine";

const compiler = new Compiler();
```

## Methods

### `execute(source_text: string, source_type?: string, source_path?: string): string`

Parses `source_text` and serializes the AST to ESTree-compatible JSON string format.

- **`source_text`**: The JavaScript or TypeScript source code string to parse.
- **`source_type`** _(optional)_: Language or extension override (e.g. `"ts"`, `"tsx"`, `"js"`).
- **`source_path`** _(optional)_: File path used to infer source type if `source_type` is omitted (defaults to `"index.js"`).

#### Usage Example

```ts
import { Compiler } from "@alloy-ts/engine";

const compiler = new Compiler();
const jsonAst = compiler.execute("const x: number = 42;", "ts", "app.ts");
const ast = JSON.parse(jsonAst);
console.log(ast.type); // "Program"
```

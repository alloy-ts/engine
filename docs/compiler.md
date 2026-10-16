# Compiler API Reference

`Compiler` provides source code parsing and ESTree JSON transformation using `oxc_parser` and `oxc_estree`.

## Constructor

### `new Compiler()`
Creates a new compiler instance.

## Instance Methods

### `execute(sourceText: string, sourceType?: string, sourcePath?: string): string`
Parses `sourceText` and returns a compact ESTree JSON string representation of the parsed AST program.

#### Parameters
- `sourceText`: JavaScript / TypeScript source code to parse/transform.
- `sourceType`: Optional file extension or module type (`"js"`, `"jsx"`, `"ts"`, `"tsx"`).
- `sourcePath`: Optional path of the source file (defaults to `"index.js"`).

#### Throws
Throws an error containing diagnostic messages if syntax errors occur.

## Example

```ts
import { Compiler } from "@alloy-ts/engine";

const compiler = new Compiler();
const estreeJson = compiler.execute("const a = 10;", "js", "app.js");
const ast = JSON.parse(estreeJson);
console.log(ast.type); // "Program"
```

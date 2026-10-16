# Compiler API Documentation

The `Compiler` class provides a native NAPI wrapper around the `oxc` parser pipeline to compile/transform source text into an ESTree-compatible serialized JSON AST string.

## Class: `Compiler`

### Constructor

#### `new Compiler()`

Constructs a new `Compiler` instance.

```ts
import { Compiler } from "@alloy-ts/engine";

const compiler = new Compiler();
```

---

### Methods

#### `execute(sourceText, sourceType?, sourcePath?)`

Parses and compiles `sourceText` into a serialized ESTree JSON string.

##### Parameters

- `sourceText` (`string`): The JavaScript or TypeScript source code to compile.
- `sourceType` (`string | undefined`): Optional language/source type override (e.g., `"js"`, `"ts"`, `"tsx"`, `"jsx"`).
- `sourcePath` (`string | undefined`): Optional file path used for source type inference and error reporting. Defaults to `"index.js"`.

##### Returns

- `string`: Compact ESTree JSON string representing the parsed `Program`.

##### Throws

- Throws an error containing diagnostic error messages if syntax errors or parse failures occur.

##### Example

```ts
import { Compiler } from "@alloy-ts/engine";

const compiler = new Compiler();

// Basic usage
const astJson = compiler.execute("const x = 42;");
const ast = JSON.parse(astJson);
console.log(ast.type); // "Program"

// With TSX source path
const tsxAstJson = compiler.execute(
  "const App = () => <h1>Hello</h1>;",
  "tsx",
  "src/App.tsx"
);
```

# Compiler

The `Compiler` class provides NAPI bindings around oxc's compilation and parsing pipeline, transforming JavaScript and TypeScript source text into ESTree-compatible AST JSON strings.

## Constructor

```ts
import { Compiler } from "@alloy-ts/engine";

const compiler = new Compiler();
```

---

## Methods

### `execute(sourceText: string, sourceType?: string, sourcePath?: string): string`

Compiles or parses `sourceText` into an ESTree AST JSON string.

#### Parameters

- `sourceText` (`string`): The source code to compile.
- `sourceType` (`string`, optional): Language override or extension (e.g. `"js"`, `"jsx"`, `"ts"`, `"tsx"`).
- `sourcePath` (`string`, optional): File path used for language inference and diagnostic reporting. Defaults to `"index.js"`.

#### Returns

A string containing the ESTree-compatible JSON representation of the parsed AST (`Program`).

#### Errors

Throws an error containing diagnostic messages if parsing/compilation encounters syntax errors or panics.

```ts
import { Compiler } from "@alloy-ts/engine";

const compiler = new Compiler();

// Compile TypeScript / TSX source
const astJson = compiler.execute(
  "const App: React.FC = () => <div />;",
  "tsx",
  "src/App.tsx"
);

const program = JSON.parse(astJson);
console.log(program.type); // "Program"
```

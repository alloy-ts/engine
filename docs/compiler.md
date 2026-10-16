# Compiler API

The `Compiler` class provides NAPI bindings to oxc's parser and AST serializer, compiling JavaScript and TypeScript source code into ESTree-compatible JSON strings.

## Usage

```ts
import { Compiler } from "@alloy-ts/engine";

const compiler = new Compiler();

// Basic compilation
const result = compiler.execute("const x = 123;", "js", "index.js");

// TypeScript / TSX compilation
const tsxResult = compiler.execute("const App = () => <div />;", "tsx", "src/App.tsx");
```

## API Reference

### `Compiler`

#### Constructor

```ts
new Compiler();
```

Creates a new `Compiler` instance.

#### Methods

##### `execute(sourceText: string, sourceType?: string | null, sourcePath?: string | null): string`

Compiles `sourceText` and returns the serialized ESTree AST JSON string.

- **`sourceText`** (`string`): The source code to compile.
- **`sourceType`** (`string | null`, optional): Language or extension override (e.g. `"js"`, `"jsx"`, `"ts"`, `"tsx"`, `"dts"`). Inferred from `sourcePath` when omitted.
- **`sourcePath`** (`string | null`, optional): File path used for language inference and diagnostic reporting. Defaults to `"index.js"`.

- **Returns:** `string` — Compact ESTree JSON representation of the AST.
- **Throws:** `Error` — If syntax or fatal parse errors occur during compilation.

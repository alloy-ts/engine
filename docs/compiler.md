# `Compiler` API Reference

The `Compiler` class provides NAPI bindings to the Oxc parser and AST serializer, enabling fast JavaScript/TypeScript parsing and AST serialization from Node.js.

## Constructor

```ts
const compiler = new Compiler();
```

Creates a new instance of the Oxc `Compiler`.

## Methods

### `execute(sourceText: string, sourceType?: string, sourcePath?: string): string`

Parses and compiles `sourceText` into an ESTree-compatible JSON AST string.

- **Parameters:**
  - `sourceText` (`string`): The JavaScript or TypeScript source code to parse/compile.
  - `sourceType` (`string | undefined`): Optional source type or extension override (e.g. `"ts"`, `"tsx"`, `"jsx"`).
  - `sourcePath` (`string | undefined`): Optional file path used for inferring language/extension (defaults to `"index.js"`).
- **Returns:**
  - `string`: ESTree JSON string representing the parsed AST program.
- **Throws:**
  - Throws an error containing joined diagnostic messages if parsing fails or panics.

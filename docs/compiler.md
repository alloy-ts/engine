# Compiler API Documentation

The `Compiler` class provides an oxc-backed parser/compiler for AST serialization and transformation.

## Constructor

```ts
const compiler = new Compiler();
```

## Methods

- `execute(sourceText: string, sourceType?: string, sourcePath?: string): string`
  Executes parsing and ESTree AST serialization for `sourceText`.
  - `sourceText`: JavaScript or TypeScript code.
  - `sourceType`: Optional source type override (e.g. `"ts"`, `"tsx"`, `"module"`).
  - `sourcePath`: Optional file path used for inference.
    Returns the ESTree JSON string representation or throws on syntax/parse diagnostics.

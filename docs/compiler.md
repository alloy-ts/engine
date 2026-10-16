# Compiler API Documentation

The `Compiler` class provides an oxc-backed parser and ESTree AST serializer.

## Constructor

```ts
import { Compiler } from "@alloy-ts/engine";

const compiler = new Compiler();
```

---

## Methods

### `execute`

```ts
compiler.execute(
  sourceText: string,
  sourceType?: string,
  sourcePath?: string
): string
```

Parses JavaScript or TypeScript source code using oxc and returns the ESTree AST serialized as a compact JSON string.

#### Parameters
- `sourceText` (`string`): The JS/TS source code to compile/parse.
- `sourceType` (`string | undefined`): Optional source type extension or language specifier (e.g., `"ts"`, `"tsx"`, `"js"`, `"jsx"`).
- `sourcePath` (`string | undefined`): Optional virtual or real file path (defaults to `"index.js"` if omitted) used for language and module kind inference.

#### Returns
- `string`: Serialized ESTree JSON representation of the parsed program (`Program` AST node).

#### Errors
- Throws an error containing diagnostic messages if parsing fails or contains syntax errors.

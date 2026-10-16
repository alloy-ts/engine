# Compiler Documentation

`Compiler` provides NAPI bindings to oxc parsing and ESTree serialization pipeline.

## Overview

The `Compiler` class compiles / parses source text into ESTree-compatible JSON strings using oxc's parser and AST serializer.

## API Reference

### Constructor

```ts
import { Compiler } from "@alloy-ts/engine";

const compiler = new Compiler();
```

Creates a new `Compiler` instance.

### Methods

#### `execute(sourceText: string, sourceType?: string, sourcePath?: string): string`

Parses `sourceText` and returns the ESTree JSON string representation of the parsed AST.

- `sourceText`: The source code string to compile/parse.
- `sourceType`: Optional language/source-type specifier (e.g. `"ts"`, `"tsx"`, `"js"`).
- `sourcePath`: Optional file path used to infer source type if `sourceType` is omitted (defaults to `"index.js"`).

```ts
const compiler = new Compiler();
const estreeJson = compiler.execute("const x = 42;", undefined, "app.js");
console.log(estreeJson);
```

Throws an error containing diagnostic messages if parsing fails or contains syntax errors.

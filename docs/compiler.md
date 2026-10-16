# Compiler

`Compiler` is a high-performance N-API wrapper around the OXC parser and AST serializer. It parses JavaScript/TypeScript source code and returns the ESTree-compliant JSON AST representation.

## Usage Example

```ts
import { Compiler } from '@alloy-ts/engine'

const compiler = new Compiler()

// Basic JavaScript compilation / AST parsing
const astJson = compiler.execute('const x = 42;')
const ast = JSON.parse(astJson)
console.log(ast.type) // "Program"

// Parsing TypeScript code with custom source path
const tsAstJson = compiler.execute('const a: number = 100;', 'ts', 'app.ts')
```

## API Reference

### `new Compiler()`

Constructs a new `Compiler` instance.

### `execute(sourceText: string, sourceType?: string, sourcePath?: string): string`

Parses `sourceText` and returns a stringified JSON ESTree representation of the parsed program.

- **`sourceText`** (`string`): The source code string to compile/parse.
- **`sourceType`** (`string | undefined`): Optional source type extension or language variant (e.g., `'js'`, `'jsx'`, `'ts'`, `'tsx'`).
- **`sourcePath`** (`string | undefined`): Optional file path used for source type detection and diagnostics (defaults to `'index.js'`).

#### Throws

Throws an `Error` if `sourceText` contains syntax errors or fails parsing.

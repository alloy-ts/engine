# Loader API Reference

`Loader` handles module resolution and loading for the engine runtime.

## Constructor

### `new Loader(options?: LoaderOptions)`

#### `LoaderOptions`
- `baseDir?: string`: Base directory used to resolve module specifiers.

## Instance Methods

### `resolve(base: string, name: string): string`
Resolves a module specifier `name` relative to `base` file path or configured `baseDir`.

## Example

```ts
import { Loader } from "@alloy-ts/engine";

const loader = new Loader({ baseDir: "/app" });
const resolvedPath = loader.resolve("/app/src/index.js", "./utils");
console.log(resolvedPath);
```

# Module Loaders

The `FileResolver` and `ScriptLoader` classes provide module resolution and loading bindings for the QuickJS engine.

## FileResolver

Resolves module specifiers to file system paths.

```ts
import { FileResolver } from "@alloy-ts/engine";

const resolver = new FileResolver();
resolver.add_path("./modules");
```

### Methods

- **`add_path(path: string): void`**: Adds a base search path for file resolution.

## ScriptLoader

Loads module source code from disk.

```ts
import { ScriptLoader } from "@alloy-ts/engine";

const loader = new ScriptLoader();
```

## Runtime Integration

Register default file resolution and loading on an `Engine` instance:

```ts
import { Engine } from "@alloy-ts/engine";

const engine = new Engine();
engine.set_file_loader();
```

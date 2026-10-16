# Loader API

Classes for configuring QuickJS module resolution and loading.

## Resolvers & Loaders

### `NativeFileResolver`

Resolves module names against file system search paths.

```ts
import { NativeFileResolver } from "@alloy-ts/engine";

const resolver = new NativeFileResolver();
resolver.add_path("./modules");
```

### `NativeBuiltinResolver`

Resolves built-in module names.

```ts
import { NativeBuiltinResolver } from "@alloy-ts/engine";

const builtinResolver = new NativeBuiltinResolver(["my_module"]);
```

### `NativeScriptLoader`

Loads JavaScript source files from the file system.

```ts
import { NativeScriptLoader } from "@alloy-ts/engine";

const scriptLoader = new NativeScriptLoader();
```

### Usage with Engine

```ts
import { Engine, NativeFileResolver, NativeScriptLoader } from "@alloy-ts/engine";

const engine = new Engine();
const resolver = new NativeFileResolver();
resolver.add_path(".");
const loader = new NativeScriptLoader();

engine.set_loader(resolver, loader);
```

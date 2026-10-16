# Loader API

The `Loader` class configures module resolution and loading for QuickJS engines and runtimes.

## Overview

```ts
import { Engine, Loader } from "@alloy-ts/engine";

const engine = new Engine();
const loader = new Loader();

// Add filesystem search directory for module imports
loader.add_search_path("./modules");

// Register in-memory virtual built-in module
loader.add_builtin("my_builtin", "export const message = 'hello from builtin';");

engine.set_loader(loader);

const ctx = engine.create_context();
const mod = ctx.eval_module("import { message } from 'my_builtin'; export { message };");
console.log(mod.message); // 'hello from builtin'
```

## Methods

- **`add_search_path(path: string): void`**
  Adds a filesystem path to search when resolving module import specifiers.

- **`add_builtin(name: string, source: string): void`**
  Registers an in-memory virtual module with a specifier name and ES module source code.

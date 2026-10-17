# Loader API Documentation

The `Loader` class allows registering JavaScript module sources for resolution and importing within QuickJS contexts.

## Overview

```ts
import { Loader, Engine } from "@alloy-ts/engine";

const loader = new Loader();
loader.add("math", "export const add = (a, b) => a + b;");

const engine = new Engine(loader);
const ctx = engine.createContext();
ctx.evalModule('import { add } from "math"; globalThis.result = add(3, 4);');
```

## Methods

- `add(name: string, sourceText: string)`: `void`
  Registers (or updates) a module source under the specified module name.

- `remove(name: string)`: `boolean`
  Removes a registered module by name. Returns `true` if the module was found and removed.

- `has(name: string)`: `boolean`
  Checks whether a module with the given name is registered.

- `keys()`: `string[]`
  Returns an array of all registered module specifiers.

- `clear()`: `void`
  Removes all registered modules from the loader registry.

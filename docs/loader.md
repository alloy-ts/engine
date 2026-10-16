# Loader (`docs/loader.md`)

The `Loader` class provides custom ES module resolution and source loading for QuickJS in `@alloy-ts/engine`.

## Overview

```ts
import { Engine, Loader } from "@alloy-ts/engine";

const loader = new Loader();
loader.add("math", "export const add = (a, b) => a + b;");

const engine = new Engine(loader);
const ctx = engine.create_context();
const res = ctx.eval_module('import { add } from "math"; export const result = add(3, 4);');
// { result: 7 }
```

## Constructor

### `new Loader()`

Creates a new empty module loader instance.

---

## Methods

#### `loader.add(name: string, source: string): void`

Registers or replaces a module with `name` and module source text `source`.

```ts
loader.add("./utils", "export const log = (msg) => console.log(msg);");
```

#### `loader.remove(name: string): boolean`

Removes the registered module `name`. Returns `true` if it existed, `false` otherwise.

#### `loader.has(name: string): boolean`

Returns `true` if a module with `name` is registered in this loader.

#### `loader.keys(): string[]`

Returns an array of all registered module specifier names.

#### `loader.clear(): void`

Removes all registered modules from this loader.

# Loader Documentation

The `Loader` class manages host-provided ES module sources for resolution and import in QuickJS contexts.

## Overview

In QuickJS, module import resolution (`import ... from "specifier"`) executes synchronously. The `Loader` class maintains a map of module specifiers to module source strings. When an `Engine` is constructed with a `Loader` (or when `engine.setLoader(loader)` is called), any context created by that engine can import the registered modules.

---

## Constructor

### `new Loader()`

Creates a new empty `Loader` registry.

---

## Methods

### `add(name: string, source: string): void`

Registers an ES module under the given specifier `name`.

- **Parameters**:
  - `name`: Module specifier string (e.g. `"math"`, `"@utils/helpers"`).
  - `source`: ES module JavaScript source code (containing `export` statements).

```ts
const loader = new Loader();
loader.add("math", "export const add = (a, b) => a + b;");
```

### `has(name: string): boolean`

Returns `true` if a module with the specifier `name` is registered in the loader.

### `keys(): string[]`

Returns an array of all registered module specifier names.

### `remove(name: string): boolean`

Removes the module registered under `name`. Returns `true` if a module was found and removed.

### `clear(): void`

Removes all registered modules from the loader.

---

## Integration Example

```ts
import { Engine, Loader } from "@alloy-ts/engine";

const loader = new Loader();
loader.add("constants", "export const PI = 3.14159;");

const engine = new Engine(loader);
const ctx = engine.createContext();

ctx.eval_module(`
  import { PI } from "constants";
  globalThis.area = (r) => PI * r * r;
`);

expect(ctx.eval("globalThis.area(2)")).toBeCloseTo(12.56636);
```

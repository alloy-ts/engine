# Loader API Documentation

The `Loader` class manages host-provided module source code for ES module resolution in QuickJS.

## Usage

```ts
import { Engine, Loader } from "@alloy-ts/engine";

const loader = new Loader();
loader.add("math", "export const add = (a, b) => a + b;");

const engine = new Engine(loader); // or engine.set_loader(loader)
const ctx = engine.create_context();
ctx.eval_module('import { add } from "math"; globalThis.res = add(2, 3);');
ctx.eval("globalThis.res"); // 5
```

---

## Methods

- `loader.add(name: string, source: string)`: Registers a module with its name/specifier and JS source text.
- `loader.remove(name: string)`: Removes a previously registered module.
- `loader.has(name: string)`: Returns `boolean` indicating if a module is registered.
- `loader.keys()`: Returns an array of registered module names/specifiers.
- `loader.clear()`: Clears all registered modules.

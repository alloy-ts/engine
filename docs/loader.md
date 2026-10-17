# Loader

The `Loader` class manages in-memory module registration and resolution for QuickJS ES module imports (`eval_module`).

## Constructor

```ts
import { Loader } from "@alloy-ts/engine";

const loader = new Loader();
```

---

## Methods

### `add(name: string, source: string): void`

Registers or replaces an in-memory ES module source string under `name`.

```ts
loader.add("math", "export const add = (a, b) => a + b;");
```

---

### `remove(name: string): boolean`

Removes a registered module by `name`. Returns `true` if the module existed.

```ts
const existed = loader.remove("math");
```

---

### `has(name: string): boolean`

Returns `true` if a module with `name` is currently registered.

```ts
if (loader.has("math")) {
  console.log("Module math is ready");
}
```

---

### `keys(): string[]`

Returns an array of registered module names.

```ts
const moduleNames = loader.keys(); // ["math"]
```

---

### `clear(): void`

Removes all registered modules from the loader registry.

```ts
loader.clear();
```

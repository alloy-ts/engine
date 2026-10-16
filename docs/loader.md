# Loader

The `Loader` class provides a host-driven module resolver and loader for ES modules imported within QuickJS contexts.

## Overview

In QuickJS, module import resolution happens synchronously during code evaluation. `Loader` allows host code to pre-register ES module source code by specifier name so that `import` statements inside QuickJS contexts resolve seamlessly.

## Constructor

```ts
import { Loader } from "@alloy-ts/engine";

const loader = new Loader();
```

## API Reference

### `loader.add(name: string, sourceText: string): void`

Registers a module specifier and its source code string.

```ts
loader.add("utils", "export const add = (a, b) => a + b;");
```

### `loader.has(name: string): boolean`

Checks whether a module specifier has been registered in the loader.

```ts
console.log(loader.has("utils")); // true
```

### `loader.keys(): string[]`

Returns an array of all registered module specifier names.

```ts
console.log(loader.keys()); // ["utils"]
```

### `loader.remove(name: string): boolean`

Removes a registered module by name. Returns `true` if the module was present and removed.

```ts
loader.remove("utils");
```

### `loader.clear(): void`

Clears all registered modules from the loader.

```ts
loader.clear();
```

## Usage with Engine

Pass a `Loader` instance to `new Engine(loader)` or attach it using `engine.set_loader(loader)`:

```ts
import { Engine, Loader } from "@alloy-ts/engine";

const loader = new Loader();
loader.add("constants", "export const PI = 3.14159;");

// Option 1: Pass to Engine constructor
const engine = new Engine(loader);

// Option 2: Attach via set_loader
// const engine = new Engine();
// engine.set_loader(loader);

const ctx = engine.create_context();
ctx.eval_module('import { PI } from "constants"; globalThis.pi = PI;');
console.log(ctx.eval("globalThis.pi")); // 3.14159
```

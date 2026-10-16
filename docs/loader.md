# QuickJS `Resolver` and `Loader` API Reference

Module resolution and loading in QuickJS are configured using `Resolver` and `Loader` builders.

## `Resolver`

Handles mapping import specifiers to canonical module names or file system paths.

### Constructor

```ts
const resolver = new Resolver();
```

Creates a new `Resolver` builder instance.

### Methods

#### `withFileResolver(): void`

Enables file system path resolution relative to the current working directory (`.`).

---

#### `addPath(path: string): void`

Adds a search directory path for resolving modules on disk.

---

## `Loader`

Handles loading module source code given resolved module names.

### Constructor

```ts
const loader = new Loader();
```

Creates a new `Loader` builder instance.

### Methods

#### `withScriptLoader(): void`

Enables reading and loading JavaScript/TypeScript files from the file system.

---

#### `withBuiltinLoader(): void`

Enables loading builtin modules.

# Loader and Resolver API

Module loader and resolver bindings for QuickJS.

## Classes

### `BuiltinModuleLoader`
Built-in module loader implementation.

```ts
const loader = new BuiltinModuleLoader();
```

### `BuiltinModuleResolver`
Built-in module resolver implementation.

```ts
const resolver = new BuiltinModuleResolver();
```

### `FileModuleResolver`
File-system module resolver for resolving relative/absolute file paths.

```ts
const fileResolver = new FileModuleResolver();
```

### `ScriptModuleLoader`
Script loader for loading and evaluating standalone scripts.

```ts
const scriptLoader = new ScriptModuleLoader();
```

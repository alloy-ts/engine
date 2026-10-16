# Loader API Documentation

The loader module provides NAPI bindings for module resolution and loading interfaces.

## Classes

### `BuiltinResolver`

Resolves built-in module specifiers.

```ts
const resolver = new BuiltinResolver();
```

### `BuiltinLoader`

Loads built-in modules.

```ts
const loader = new BuiltinLoader();
```

### `FileResolver`

Resolves file system path module specifiers.

```ts
const resolver = new FileResolver();
```

### `ScriptLoader`

Loads script files.

```ts
const loader = new ScriptLoader();
```

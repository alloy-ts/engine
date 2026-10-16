# Module Loaders & Resolvers API

QuickJS supports module resolution and loading through resolver and loader structures.

## Available Classes

### `JsFileResolver`

Resolves module specifiers to file system paths.

- **`new JsFileResolver()`**
  Constructs a new file resolver.

- **`add_path(path: string): void`**
  Adds a search path for resolving module imports.

### `JsScriptLoader`

Loads script files from resolved file paths.

- **`new JsScriptLoader()`**
  Constructs a new script loader.

### `JsBuiltinResolver`

Resolves native / built-in module names.

- **`new JsBuiltinResolver()`**
  Constructs a builtin module resolver.

- **`add_module(name: string): void`**
  Registers a module name as a built-in module.

### `JsBuiltinLoader`

Loads bytecodes or definitions for built-in modules.

- **`new JsBuiltinLoader()`**
  Constructs a builtin module loader.

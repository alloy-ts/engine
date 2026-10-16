# Loader Documentation

`Loader` describes module loading and resolution mechanisms in the QuickJS engine (`rquickjs::loader`).

## Overview

QuickJS supports ES module resolution and loading. Modularity relies on two key interfaces:

1. **Resolver**: Resolves module names or specifiers relative to a base path.
2. **Loader**: Loads module source code or bytecode into the QuickJS runtime context.

## Resolver Interface

The resolver normalizes and resolves specifiers (e.g. `./math.js`, `foo`):

```rust
pub trait Resolver {
  fn resolve<'js>(
    &mut self,
    ctx: &Ctx<'js>,
    base: &str,
    name: &str,
    attributes: Option<ImportAttributes<'js>>,
  ) -> Result<String, Error>;
}
```

## Loader Interface

The loader retrieves and returns the module for evaluation:

```rust
pub trait Loader {
  fn load<'js>(
    &mut self,
    ctx: &Ctx<'js>,
    name: &str,
    attributes: Option<ImportAttributes<'js>>,
  ) -> Result<Module<'js>, Error>;
}
```

## Built-in Loaders & Resolvers

- **`BuiltinResolver` / `BuiltinLoader`**: Native / built-in modules.
- **`FileResolver` / `ScriptLoader`**: Local filesystem resolution and script execution.
- **`ModuleLoader`**: ES module compilation and loading.

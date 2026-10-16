# Module Loader API

QuickJS supports ES module resolution and loading. The module loader interface allows customizing module specifier resolution (`Resolver`) and source loading (`Loader`).

## Resolvers and Loaders

- **Resolver:** Maps an imported module specifier (e.g., `"./math.js"` or `"lodash"`) to a canonical module name or URL.
- **Loader:** Reads module source code or bytecode for a given resolved specifier.

## QuickJS Native Module Features

In `rquickjs`, module loading is configured on the `Runtime` instance using:

```rust
runtime.set_loader(resolver, loader);
```

### Module Evaluation

Modules loaded into a `QuickJSContext` are evaluated asynchronously or synchronously depending on top-level await features.

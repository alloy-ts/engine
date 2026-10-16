# Loader

`Loader` provides module loader configuration for QuickJS runtime module resolution.

## Overview

A `Loader` configures how module specifiers and paths are resolved when importing ESM modules.

## Usage

```ts
import { Loader } from "@alloy-ts/engine";

const loader = new Loader("/path/to/modules");
console.log(loader.get_base_path()); // "/path/to/modules"
```

## Methods

- **`constructor(base_path?: string)`**: Creates a new module loader with an optional base directory.
- **`get_base_path(): string | null`**: Returns the configured base directory path.

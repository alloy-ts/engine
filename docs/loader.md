# Loader

Configures ES module resolution paths and registers builtin modules for an `Engine` runtime instance.

## Usage

```ts
import { Engine, Loader } from "@alloy-ts/engine";

const engine = new Engine();
const loader = new Loader();

// Add search path for ES module imports
loader.addPath("./modules");

// Register virtual/builtin module in memory
loader.addModule("virtual:math", "export const add = (a, b) => a + b;");

// Apply loader configuration to engine runtime
engine.setLoader(loader);
```

## Methods

### `addPath(path: string): void`

Adds a directory path to the file module resolver search list.

### `addModule(name: string, source: string): void`

Registers a builtin module by name with virtual source code.

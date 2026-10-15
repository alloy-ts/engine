# Loader

`Loader` provides module loading and resolution capabilities for the embedded QuickJS `Engine`.

## Usage

```ts
import { Engine, Loader } from "@alloy-ts/engine";

const engine = new Engine();
const loader = new Loader();

engine.setLoader(loader);
```

## Constructor

### `new Loader()`

Constructs a new module loader configured with builtin and script loaders and file resolvers.

# Engine

The `Engine` class wraps the JS execution context and runtime state.

## Usage

```ts
import { Engine } from "@alloy-ts/engine";

const engine = new Engine();
engine.create_string("hello");
engine.create_array(3);
```

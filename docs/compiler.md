# Compiler API

The `Compiler` class provides OXC parser and ESTree serialization bindings.

## Usage

```ts
import { Compiler } from "@alloy-ts/engine";

const compiler = new Compiler();
const astJson = compiler.execute("const x = 1;", "ts", "app.ts");
const ast = JSON.parse(astJson);
```

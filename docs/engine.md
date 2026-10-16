# JsEngine API Reference

`JsEngine` provides JavaScript execution and compilation capabilities via native Node-API bindings.

## Overview

The engine exposes native classes for script execution, compilation, context management, and module resolution:

- **`Compiler`**: Source code transformer backed by `oxc_parser` and `oxc_estree`.
- **`Engine`**: QuickJS execution engine/runtime wrapper (`rquickjs::Runtime`).
- **`Context`**: Individual JavaScript execution context (`rquickjs::Context`).
- **`Loader`**: Module loader and specifier resolver.

---

## API Reference

### `Compiler`

```ts
class Compiler {
  constructor();
  execute(sourceText: string, sourceType?: string, sourcePath?: string): string;
}
```

#### Example

```ts
import { Compiler } from "@alloy-ts/engine";

const compiler = new Compiler();
const estreeJson = compiler.execute("const x: number = 42;", "ts", "app.ts");
```

---

### `Engine`

```ts
class Engine {
  constructor();
  setMemoryLimit(limitBytes: number): void;
  setMaxStackSize(limitBytes: number): void;
  setGcThreshold(thresholdBytes: number): void;
  runGc(): void;
  isJobPending(): boolean;
  execPendingJob(): boolean;
}
```

#### Example

```ts
import { Engine, Context } from "@alloy-ts/engine";

const engine = new Engine();
engine.setMemoryLimit(1024 * 1024 * 64); // 64MB limit
const context = Context.full(engine);
```

---

### `Context`

```ts
class Context {
  static base(engine: Engine): Context;
  static full(engine: Engine): Context;
  eval(source: string): string;
}
```

#### Example

```ts
import { Engine, Context } from "@alloy-ts/engine";

const engine = new Engine();
const ctx = Context.full(engine);
const result = ctx.eval("1 + 1"); // "2"
```

---

### `Loader`

```ts
class Loader {
  constructor(options?: LoaderOptions);
  resolve(base: string, name: string): string;
}

interface LoaderOptions {
  baseDir?: string;
}
```

#### Example

```ts
import { Loader } from "@alloy-ts/engine";

const loader = new Loader({ baseDir: "/project/root" });
const resolved = loader.resolve("/project/root/src/index.js", "./utils");
```

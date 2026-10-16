# JsEngine API Reference

`JsEngine` provides JavaScript execution and compilation capabilities via native Node-API bindings.

## Overview

The engine exposes native classes for script execution, compilation, context management, and module resolution:

- **`Compiler`**: Source code transformer backed by `oxc_compiler`.
- **`Engine`**: QuickJS execution engine/runtime wrapper (`rquickjs::Runtime`).
- **`Context`**: Individual JavaScript execution context (`rquickjs::Context`).
- **`Loader`**: Module loader and specifier resolver.

---

## API Reference

### `Compiler`

```ts
class Compiler {
  constructor();
  execute(sourceText: string, options?: CompilerOptions): string;
}

interface CompilerOptions {
  filename?: string;
  lang?: "js" | "jsx" | "ts" | "tsx" | "dts";
  sourceType?: "script" | "module" | "commonjs" | "unambiguous";
}
```

#### Example

```ts
import { Compiler } from "@alloy-ts/engine";

const compiler = new Compiler();
const code = compiler.execute("const x: number = 42;", { filename: "app.ts" });
```

---

### `Engine`

```ts
class Engine {
  constructor();
  set_memory_limit(limitBytes: number): void;
  set_max_stack_size(limitBytes: number): void;
  set_gc_threshold(thresholdBytes: number): void;
  run_gc(): void;
  is_job_pending(): boolean;
  execute_pending_job(): boolean;
}
```

#### Example

```ts
import { Engine, Context } from "@alloy-ts/engine";

const engine = new Engine();
engine.set_memory_limit(1024 * 1024 * 64); // 64MB limit
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

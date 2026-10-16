# JsEngine API Reference

`JsEngine` provides JavaScript execution and compilation capabilities via native Node-API bindings.

## Overview

The engine exposes native classes for script execution, compilation, context management, and module resolution:

- **`Compiler`**: Source code transformer backed by `oxc_compiler`.
- **`Runtime`**: QuickJS execution runtime wrapper (`rquickjs::Runtime`).
- **`Context`**: Individual JavaScript execution context (`rquickjs::Context`).
- **`Loader`**: Module loader and specifier resolver.
- **`Engine`**: Node-API (`napi::Engine`) context handle wrapper.

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

### `Runtime`

```ts
class Runtime {
  constructor();
  setMemoryLimit(limitBytes: number): void;
  setMaxStackSize(limitBytes: number): void;
  setGcThreshold(thresholdBytes: number): void;
  runGc(): void;
  isJobPending(): boolean;
  executePendingJob(): boolean;
}
```

#### Example

```ts
import { Runtime, Context } from "@alloy-ts/engine";

const runtime = new Runtime();
runtime.setMemoryLimit(1024 * 1024 * 64); // 64MB limit
const context = Context.full(runtime);
```

---

### `Context`

```ts
class Context {
  static base(runtime: Runtime): Context;
  static full(runtime: Runtime): Context;
  eval(source: string): string;
}
```

#### Example

```ts
import { Runtime, Context } from "@alloy-ts/engine";

const rt = new Runtime();
const ctx = Context.full(rt);
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

---

### `Engine`

```ts
class Engine {
  constructor();
  is_disposed(): boolean;
  create_array(len: number): unknown;
  create_string(value: string): unknown;
  create_int32(int: number): unknown;
  create_double(value: number): unknown;
  has_global(name: string): boolean;
  get_napi_version(): number;
  make_object(): object;
  set_instance_data(data: string): void;
  get_instance_data(): string | null;
}
```

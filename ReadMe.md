# JS Engine

A JavaScript execution engine with a native (Rust) core as a Node-API addon backed by QuickJS (`rquickjs`). The Rust side is compiled to `alloy-engine.<platform>.node` via [`@napi-rs/cli`](https://napi.rs), and the TypeScript side is bundled by [vite-plus](https://viteplus.dev).

The addon exposes high-level NAPI bindings for QuickJS (`Engine`, `Context`, `Loader`, `WeakRuntime`), `Parser` (oxc-backed), `Global`, and `add`.

## Layout

| Path             | Role                                                                |
| ---------------- | ------------------------------------------------------------------- |
| `src/lib.rs`     | Crate root: module declarations and the `add` binding               |
| `src/context.rs` | `Context` — NAPI class wrapping a QuickJS execution realm (`Ctx`)   |
| `src/engine.rs`  | `Engine` — NAPI class wrapping an embedded QuickJS `Runtime`        |
| `src/loader.rs`  | `Loader` — NAPI class wrapping custom ES module resolver and loader |
| `src/global.rs`  | `Global` — NAPI class providing helpers for the host `globalThis`   |
| `src/parser.rs`  | `Parser` — NAPI wrapper around `oxc_parser` (ESTree JSON output)    |
| `src/main.ts`    | vite-plus pack entry; re-exports the native bindings                |
| `src/*.test.ts`  | Unit tests (run by `vp test`)                                       |
| `build.ts`       | Drives `@napi-rs/cli` to build the `.node` addon into `dist/`       |
| `vite.config.ts` | `pack` / `test` / `lint` / `fmt` configuration                      |
| `Cargo.toml`     | Rust crate `alloy-engine-native` (`cdylib`)                         |

## NAPI surface

### `Engine`

Wraps both the host N-API context (`napi::Env`) and an embedded QuickJS `Runtime`. Can be constructed directly or with an optional module `Loader`.

```ts
import { Engine, Loader } from "@alloy-ts/engine";

const loader = new Loader();
loader.add("math", "export const add = (a, b) => a + b;");

const engine = new Engine(loader);
const ctx = engine.create_context();
const res = ctx.eval_module('import { add } from "math"; export const sum = add(2, 3);');
// { sum: 5 }

// QuickJS Runtime methods
engine.set_memory_limit(1024 * 1024);
engine.run_gc();
engine.execPendingJob();

// N-API Env methods
engine.create_string("hello");
engine.create_array(3);
engine.has_global("Math");
```

Available QuickJS Runtime methods: `create_context()`, `set_loader(loader)`, `weak()`, `set_memory_limit(limit)`, `set_max_stack_size(limit)`, `set_gc_threshold(threshold)`, `set_info(info)`, `run_gc()`, `memory_usage()`, `is_job_pending()`, `execute_pending_job()`, `execPendingJob()`.

Available N-API Env methods: `is_disposed()`, `create_array(len)`, `create_string(val)`, `create_int32(val)`, `create_double(val)`, `has_global(name)`, `get_napi_version()`, `make_object()`, `set_instance_data(data)`, `get_instance_data()`.

### `Context`

Wraps a QuickJS execution context. Created either standalone (`new Context()`) or from an `Engine` (`engine.create_context()`).

```ts
import { Context } from "@alloy-ts/engine";

const ctx = new Context();
ctx.eval("1 + 1"); // 2
ctx.eval_module("export const x = 42;"); // { x: 42 }
ctx.json_parse('{"a": 1}'); // { a: 1 }
```

Available methods: `eval(source)`, `eval_module(source)`, `json_parse(json)`, `global_keys()`, `run_gc()`, `execute_pending_job()`, `has_exception()`, `catch()`.

### `Loader`

Custom ES module loader and resolver.

```ts
import { Loader } from "@alloy-ts/engine";

const loader = new Loader();
loader.add("foo", "export const val = 123;");
loader.has("foo"); // true
loader.keys(); // ["foo"]
```

Available methods: `add(name, source)`, `remove(name)`, `has(name)`, `keys()`, `clear()`.

### `Parser`

Wraps `oxc_parser`. Construct it with source text and an optional file path or extension; then call `parser(options?)` for a full `Program` or `parse_expression(options?)` for an expression. Returns ESTree-compatible JSON.

```ts
import { Parser } from "@alloy-ts/engine";

const parser = new Parser("const x = 1;", "src/app.tsx");
const { program, errors, fatalError } = parser.parser();
```

### `add(left: i32, right: i32) -> i32`

Helper function binding from `src/lib.rs`.

```ts
import { add } from "@alloy-ts/engine";

add(2, 3); // 5
```

## Development

- Configure local hooks:

```bash
npm run prepare
```

- Install dependencies:

```bash
npm install
```

- Run the unit tests:

```bash
npm test
```

- Build the native addon:

```bash
npm run build:napi
```

- Code formatting & linting:

```bash
npm run check
```

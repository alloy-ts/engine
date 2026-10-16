# JS Engine

A JavaScript execution engine with a native (Rust) core as a Node-API addon using
QuickJS (`rquickjs`) and OXC. The Rust side is compiled to `alloy-engine.<platform>.node` via
[`@napi-rs/cli`](https://napi.rs), and the TypeScript side is bundled by
[vite-plus](https://viteplus.dev).

Today the addon exposes bindings for: `add`, `Parser` (oxc-backed), `Compiler` (oxc-backed), `Engine`, `Context`, `Runtime`, `WeakRuntime`, and `Loader`.

## Layout

| Path              | Role                                                                 |
| ----------------- | -------------------------------------------------------------------- |
| `src/lib.rs`      | Crate root: module declarations and the `add` binding                |
| `src/compiler.rs` | `Compiler` — NAPI wrapper around OXC compilation/transformation      |
| `src/context.rs`  | `Context` — NAPI bindings for QuickJS execution contexts (`Context`) |
| `src/engine.rs`   | `Engine` — Unifies N-API context (`napi::Env`) & QuickJS `Runtime`   |
| `src/runtime.rs`  | `Runtime` & `WeakRuntime` — Standalone QuickJS runtime bindings      |
| `src/loader.rs`   | `Loader` — QuickJS module resolution and loader configuration        |
| `src/parser.rs`   | `Parser` — NAPI wrapper around `oxc_parser` (ESTree JSON out)        |
| `src/main.ts`     | vite-plus pack entry; re-exports the native bindings                 |
| `src/*.test.ts`   | Unit tests (run by `vp test`)                                        |
| `build.ts`        | Drives `@napi-rs/cli` to build the `.node` addon into `dist/`        |
| `vite.config.ts`  | `pack` / `test` / `lint` / `fmt` configuration                       |
| `Cargo.toml`      | Rust crate `alloy-engine-native` (`cdylib`)                          |

## NAPI surface

napi-rs exposes methods under their Rust (`snake_case`) name unless a `js_name`
override is set, so the JS API mirrors the Rust names below.

### `add(left: i32, right: i32) -> i32`

Plain function binding from `src/lib.rs`.

```ts
import { add } from "@alloy-ts/engine";

add(2, 3); // 5
```

### `Compiler`

NAPI wrapper around the OXC compiler/parser pipeline.

```ts
import { Compiler } from "@alloy-ts/engine";

const compiler = new Compiler();
const astString = compiler.execute("const x = 1;", "js", "index.js");
```

### `Parser`

Wraps `oxc_parser`. Construct it with source text and an optional file path or
bare extension used to infer the language / module kind; then call
`parser(options?)` for a full `Program` or `parse_expression(options?)` for a
single expression. Both return ESTree-compatible JSON.

```ts
import { Parser } from "@alloy-ts/engine";

const parser = new Parser("const x = 1;", "src/app.tsx");
const { program, errors, fatalError, isFlowLanguage } = parser.parser();
```

`parser()` collects recoverable diagnostics instead of throwing (check `errors`
/ `fatalError`); `parse_expression()` throws on a syntax error.

### `Engine`

A JS class that owns both the N-API context handle and an embedded QuickJS `Runtime`.

```ts
import { Engine } from "@alloy-ts/engine";

const engine = new Engine();

// QuickJS Runtime capabilities
const ctx = engine.create_context();
ctx.eval("1 + 1"); // 2

engine.set_memory_limit(1024 * 1024 * 32);
engine.set_max_stack_size(256 * 1024);
engine.run_gc();
engine.memory_usage();
engine.is_job_pending();
engine.execPendingJob(); // alias for execute_pending_job

// Host N-API context capabilities
engine.create_string("hello"); // "hello"
engine.create_int32(42); // 42
engine.create_array(3); // [ <3 empty items> ]
engine.has_global("Math"); // true
engine.get_napi_version(); // host N-API version
engine.is_disposed(); // false while the addon is loaded
```

### `Context`

Wraps a QuickJS execution context (`rquickjs::Context`).

```ts
import { Context } from "@alloy-ts/engine";

const ctx = Context.new();
ctx.eval("1 + 1"); // 2
ctx.eval_module("export const a = 10;"); // { a: 10 }
ctx.json_parse('{"x": 42}'); // { x: 42 }
ctx.global_keys(); // ["Math", "Array", ...]
ctx.run_gc();
ctx.execute_pending_job();
```

### `Loader`

Configures module resolution and loading (file search paths and in-memory built-in modules) on a QuickJS `Engine` or `Runtime`.

```ts
import { Engine, Loader } from "@alloy-ts/engine";

const engine = new Engine();
const loader = new Loader();
loader.add_builtin("my_mod", "export const msg = 'hello';");
engine.set_loader(loader);

const ctx = engine.create_context();
ctx.eval_module("import { msg } from 'my_mod'; export { msg };"); // { msg: 'hello' }
```

## Development

- Configure local hooks:

```bash
npm run prepare
```

- Install dependencies:

```bash
vp install
```

- Run the unit tests:

```bash
vp test
```

- Build the library (native addon + bundle):

```bash
npm run build
```

- Build only the native addon:

```bash
npm run build:napi       # debug
npm run build:release    # release
```

- Code formatting:

```bash
npm run fmt
```

- Linting:

```bash
npm run lint
```

- Type check:

```bash
npm run check
```

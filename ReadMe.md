# JS Engine

A JavaScript execution engine with a native (Rust) core as a Node-API addon wrapping QuickJS via `rquickjs`. The
Rust side is compiled to `alloy-engine.<platform>.node` via
[`@napi-rs/cli`](https://napi.rs), and the TypeScript side is bundled by
[vite-plus](https://viteplus.dev).

Today the addon exposes bindings including `add`, `Parser` (oxc-backed), `Engine` / `JsEngine` (QuickJS runtime), and `Context` (QuickJS execution context).

## Layout

| Path             | Role                                                                   |
| ---------------- | ---------------------------------------------------------------------- |
| `src/lib.rs`     | Crate root: module declarations and the `add` binding                  |
| `src/parser.rs`  | `Parser` — NAPI wrapper around `oxc_parser` (ESTree JSON out)          |
| `src/engine.rs`  | `Engine` — JS class wrapping the QuickJS runtime (`rquickjs::Runtime`) |
| `src/context.rs` | `Context` — JS class wrapping `rquickjs::Context`                      |
| `src/loader.rs`  | `Loader` — QuickJS module loader configuration                         |
| `src/main.ts`    | vite-plus pack entry; re-exports the native bindings                   |
| `src/*.test.ts`  | Unit tests (run by `vp test`)                                          |
| `docs/engine.md` | API reference for `JsEngine` / `Engine`                                |
| `build.ts`       | Drives `@napi-rs/cli` to build the `.node` addon into `dist/`          |
| `vite.config.ts` | `pack` / `test` / `lint` / `fmt` configuration                         |
| `Cargo.toml`     | Rust crate `alloy-engine-native` (`cdylib`)                            |

## NAPI surface

napi-rs exposes methods under their Rust (`snake_case`) name unless a `js_name`
override is set, so the JS API mirrors the Rust names below.

### `add(left: i32, right: i32) -> i32`

Plain function binding from `src/lib.rs`.

```ts
import { add } from "@alloy-ts/engine";

add(2, 3); // 5
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

### `Engine` / `JsEngine` & `Context`

`Engine` (also exported as `JsEngine`) manages the QuickJS runtime. Pass `Engine` instances to `Context.full(engine)` to create execution contexts:

```ts
import { Context, Engine, JsEngine } from "@alloy-ts/engine";

const engine = new JsEngine();
engine.set_memory_limit(1024 * 1024 * 64); // 64MB

const ctx = Context.full(engine);
const result = ctx.eval("1 + 1"); // "2"
```

Available `Engine` methods: `set_info`, `set_memory_limit`, `set_max_stack_size`, `set_gc_threshold`, `run_gc`, `is_job_pending`, `execute_pending_job` (`execPendingJob`). See [`docs/engine.md`](docs/engine.md) for the full API reference.

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

- Run locally:

```bash
npm run dev
```

- Build the library (native addon + bundle):

```bash
npm run build
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

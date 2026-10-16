# JS Engine

A JavaScript execution engine with a native (Rust) core as a Node-API addon. The
Rust side is compiled to `alloy-engine.<platform>.node` via
[`@napi-rs/cli`](https://napi.rs), and the TypeScript side is bundled by
[vite-plus](https://viteplus.dev).

The addon exposes bindings: `add`, `Parser` (oxc-backed), `Engine` (QuickJS Runtime & N-API context), `Context` (`JSContext`), `FileResolver`, and `ScriptLoader`.

## Layout

| Path             | Role                                                           |
| ---------------- | -------------------------------------------------------------- |
| `src/lib.rs`     | Crate root: module declarations and the `add` binding          |
| `src/context.rs` | `Context` (`JSContext`) — QuickJS execution context            |
| `src/engine.rs`  | `Engine` — JS class wrapping QuickJS Runtime and N-API context |
| `src/loader.rs`  | `FileResolver` / `ScriptLoader` — QuickJS module loaders       |
| `src/parser.rs`  | `Parser` — NAPI wrapper around `oxc_parser` (ESTree JSON out)  |
| `src/main.ts`    | vite-plus pack entry; re-exports the native bindings           |
| `src/*.test.ts`  | Unit tests (run by `vp test`)                                  |
| `build.ts`       | Drives `@napi-rs/cli` to build the `.node` addon into `dist/`  |
| `vite.config.ts` | `pack` / `test` / `lint` / `fmt` configuration                 |
| `Cargo.toml`     | Rust crate `alloy-engine-native` (`cdylib`)                    |

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

### `Engine`

A JS class that owns the N-API context handle and QuickJS `Runtime`:

```ts
import { Engine } from "@alloy-ts/engine";

const engine = new Engine();
engine.set_info("engine-instance");
engine.set_memory_limit(1024 * 1024);
engine.execPendingJob();
engine.run_gc();
const mem = engine.memory_usage();
```

Available methods: `is_disposed`, `create_array`, `create_string`,
`create_int32`, `create_double`, `has_global`, `get_napi_version`,
`make_object`, `set_instance_data`, `get_instance_data`, `set_info`,
`set_memory_limit`, `set_max_stack_size`, `set_gc_threshold`, `set_dump_flags`,
`run_gc`, `is_job_pending`, `execPendingJob`, `execute_pending_job`,
`memory_usage`, `set_file_loader`.

### `Context` (`JSContext`)

QuickJS execution context:

```ts
import { Context, Engine } from "@alloy-ts/engine";

const engine = new Engine();
const ctx = Context.full(engine);
ctx.eval("1 + 1"); // 2
ctx.eval_promise("await Promise.resolve(42)"); // 42
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

- Run locally:

```bash
npm run dev
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

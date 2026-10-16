# JS Engine

A JavaScript execution engine with a native (Rust) core as a Node-API addon. The
Rust side is compiled to `alloy-engine.<platform>.node` via
[`@napi-rs/cli`](https://napi.rs), and the TypeScript side is bundled by
[vite-plus](https://viteplus.dev).

The addon exposes bindings for: `add`, `Parser` (oxc-backed), `Engine` (JS class wrapping N-API context + QuickJS Runtime), `Context` (QuickJS Context), and module loaders (`FileResolver`, `ScriptLoader`, `BuiltinResolver`, `BuiltinLoader`).

## Layout

| Path             | Role                                                           |
| ---------------- | -------------------------------------------------------------- |
| `src/lib.rs`     | Crate root: module declarations and the `add` binding          |
| `src/context.rs` | `Context` — NAPI wrapper around QuickJS `rquickjs::Context`    |
| `src/engine.rs`  | `Engine` — JS class wrapping N-API context and QuickJS Runtime |
| `src/loader.rs`  | QuickJS module resolvers and loaders                           |
| `src/parser.rs`  | `Parser` — NAPI wrapper around `oxc_parser` (ESTree JSON out)  |
| `src/main.ts`    | vite-plus pack entry; re-exports native bindings               |
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

### `Context`

NAPI wrapper around QuickJS execution context.

```ts
import { Context } from "@alloy-ts/engine";

const ctx = new Context();
ctx.eval("1 + 1"); // 2
ctx.json_parse('{"a": 1}'); // { a: 1 }
ctx.json_stringify({ b: 2 }); // '{"b":2}'
```

### `Engine`

A JS class that owns the N-API context handle and QuickJS Runtime instance.

```ts
import { Engine } from "@alloy-ts/engine";

const engine = new Engine();
engine.create_string("hello"); // "hello"
engine.create_int32(42); // 42
engine.create_array(3); // [ <3 empty items> ]
engine.has_global("Math"); // true
engine.get_napi_version(); // host N-API version

const ctx = engine.create_context();
ctx.eval("2 + 3"); // 5

engine.is_job_pending(); // false
engine.execute_pending_job(); // false
```

Available methods: `is_disposed`, `create_array`, `create_string`, `create_int32`, `create_double`, `has_global`, `get_napi_version`, `make_object`, `set_instance_data`, `get_instance_data`, `create_context`, `create_base_context`, `set_info`, `set_memory_limit`, `set_max_stack_size`, `set_gc_threshold`, `set_dump_flags`, `run_gc`, `is_job_pending`, `execute_pending_job`, `memory_usage`, `set_file_loader`, `set_builtin_loader`.

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

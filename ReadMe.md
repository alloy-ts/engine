# JS Engine

A JavaScript execution engine with a native (Rust) core as a Node-API addon. The
Rust side is compiled to `alloy-engine.<platform>.node` via
[`@napi-rs/cli`](https://napi.rs), and the TypeScript side is bundled by
[vite-plus](https://viteplus.dev).

The addon exposes bindings for: `add`, `Parser` (oxc-backed), `Engine` (a JS class
wrapping both N-API context helpers and QuickJS Runtime), and `Context` (QuickJS execution context).

## Layout

| Path             | Role                                                           |
| ---------------- | -------------------------------------------------------------- |
| `src/lib.rs`     | Crate root: module declarations and the `add` binding          |
| `src/parser.rs`  | `Parser` — NAPI wrapper around `oxc_parser` (ESTree JSON out)  |
| `src/engine.rs`  | `Engine` — JS class wrapping N-API context & QuickJS `Runtime` |
| `src/context.rs` | `Context` — JS class wrapping QuickJS `Context`                |
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

`parser()` collects recoverable diagnostics instead of throwing (check `errors`
/ `fatalError`); `parse_expression()` throws on a syntax error.

`ParseOptions` mirrors `oxc_parser::ParseOptions` — every field is optional and
falls back to oxc's default: `parse_regular_expression`,
`allow_return_outside_function`, `preserve_parens`, `allow_v8_intrinsics`,
`enable_ident_hashes`.

### `Engine`

A JS class that owns the QuickJS `Runtime` handle and provides N-API environment
helper methods as well.

```ts
import { Engine } from "@alloy-ts/engine";

const engine = new Engine();

// N-API context / environment helpers
engine.create_string("hello"); // "hello"
engine.create_int32(42); // 42
engine.create_array(3); // [ <3 empty items> ]
engine.has_global("Math"); // true
engine.get_napi_version(); // host N-API version

engine.set_instance_data("vm-state");
engine.get_instance_data(); // "vm-state"
engine.is_disposed(); // false while the addon is loaded

// QuickJS Runtime methods
engine.set_memory_limit(1024 * 1024);
engine.set_max_stack_size(256 * 1024);
engine.set_gc_threshold(512 * 1024);
engine.run_gc();
engine.is_job_pending();
engine.execute_pending_job();
engine.execPendingJob();
```

Available methods: `is_disposed`, `create_array`, `create_string`,
`create_int32`, `create_double`, `has_global`, `get_napi_version`,
`make_object`, `set_instance_data`, `get_instance_data`, `set_memory_limit`,
`set_max_stack_size`, `set_gc_threshold`, `run_gc`, `is_job_pending`,
`execute_pending_job`, `execPendingJob`.

### `Context`

A JS class wrapping QuickJS `Context` for executing JavaScript code.

```ts
import { Context } from "@alloy-ts/engine";

const ctx = new Context();
const result = ctx.eval("1 + 1"); // "2"
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

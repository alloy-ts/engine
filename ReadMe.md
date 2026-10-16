# JS Engine

A JavaScript execution engine with a native (Rust) core as a Node-API addon. The
Rust side is compiled to `alloy-engine.<platform>.node` via
[`@napi-rs/cli`](https://napi.rs), and the TypeScript side is bundled by
[vite-plus](https://viteplus.dev).

Today the addon exposes bindings: `add`, `Parser` (oxc-backed), `Engine` (wrapping N-API context & QuickJS Runtime), `Context`, `Runtime`, and `Loader`.

## Layout

| Path             | Role                                                            |
| ---------------- | --------------------------------------------------------------- |
| `src/lib.rs`     | Crate root: module declarations and the `add` binding           |
| `src/parser.rs`  | `Parser` — NAPI wrapper around `oxc_parser` (ESTree JSON out)   |
| `src/engine.rs`  | `Engine` — JS class wrapping the N-API context and QuickJS Runtime |
| `src/context.rs` | `Context` — JS class wrapping embedded QuickJS execution context |
| `src/runtime.rs` | `Runtime` / `WeakRuntime` — QuickJS runtime engine handles |
| `src/loader.rs`  | `Loader` — QuickJS module resolver and loader registry |
| `src/main.ts`    | vite-plus pack entry; re-exports the native bindings            |
| `src/*.test.ts`  | Unit tests (run by `vp test`)                                   |
| `build.ts`       | Drives `@napi-rs/cli` to build the `.node` addon into `dist/`   |
| `vite.config.ts` | `pack` / `test` / `lint` / `fmt` configuration                  |
| `Cargo.toml`     | Rust crate `alloy-engine-native` (`cdylib`)                     |

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

A JS class that owns the N-API context handle and an embedded QuickJS `Runtime`.
JS constructs it with an optional `Loader` parameter:

```ts
import { Engine } from "@alloy-ts/engine";

const engine = new Engine();
engine.create_string("hello"); // "hello"
engine.create_int32(42); // 42
engine.create_array(3); // [ <3 empty items> ]
engine.has_global("Math"); // true
engine.get_napi_version(); // host N-API version

engine.set_instance_data("vm-state");
engine.get_instance_data(); // "vm-state"

engine.is_disposed(); // false while the addon is loaded

// QuickJS runtime capabilities:
const ctx = engine.create_context();
engine.execPendingJob();
engine.run_gc();
```

Available methods: `is_disposed`, `create_array`, `create_string`,
`create_int32`, `create_double`, `has_global`, `get_napi_version`,
`make_object`, `set_instance_data`, `get_instance_data`, `create_context`,
`execPendingJob`, `execute_pending_job`, `run_gc`, `set_memory_limit`,
`set_max_stack_size`, `set_gc_threshold`, `set_info`, `memory_usage`.

### `Context`

An execution realm in QuickJS with its own global object:

```ts
import { Context } from "@alloy-ts/engine";

const ctx = new Context();
ctx.eval("1 + 1"); // 2
ctx.eval_module("export const x = 42;"); // { x: 42 }
ctx.json_parse('{"a": 1}'); // { a: 1 }
```

### `Loader`

Module registry and resolver for QuickJS runtime contexts:

```ts
import { Loader, Runtime } from "@alloy-ts/engine";

const loader = new Loader();
loader.add("math", "export const add = (a, b) => a + b;");

const rt = new Runtime(loader);
const ctx = rt.create_context();
ctx.eval_module('import { add } from "math"; export const sum = add(2, 3);');
```

#### The N-API context contract

`napi::Engine` is the handle the N-API implementation uses to persist
VM-specific state. `Engine` follows its rules:

1. **The same context must be reused for nested calls.** Every method does its
   N-API work through the handle captured at construction.
2. **It must not be cached for general reuse, nor shared between addon instances
   or Worker threads.** Each `Engine` captures its own context; the fields are
   private so it cannot leak out.
3. **It becomes invalid when the addon instance is unloaded.** `Engine::new`
   registers an environment cleanup hook that flips a disposed flag, and every
   public method refuses to use a dangling handle. `set_instance_data`'s
   finalize callback observes the same unload event.

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

## Known gaps

- `src/main.ts` does not exist yet. It is the vite-plus pack entry
  (`vite.config.ts`) and `main.test.ts` already imports `main()` from it, so it
  must be created and must re-export `add`, `Parser` and `Engine`.
- `src/lib.rs` declares `pub mod future; mod poll; mod result;` but those
  modules have no files yet, which currently blocks `cargo build`.

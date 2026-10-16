# JS Engine

A JavaScript execution engine with a native (Rust) core as a Node-API addon. The
Rust side is compiled to `alloy-engine.<platform>.node` via
[`@napi-rs/cli`](https://napi.rs), and the TypeScript side is bundled by
[vite-plus](https://viteplus.dev).

Today the addon exposes bindings for: `add`, `Parser` (oxc-backed), `Engine` (QuickJS runtime), `Context`, `Resolver`, and `Loader`.

## Layout

| Path             | Role                                                                |
| ---------------- | ------------------------------------------------------------------- |
| `src/lib.rs`     | Crate root: module declarations and `add` binding                   |
| `src/parser.rs`  | `Parser` — NAPI wrapper around `oxc_parser` (ESTree JSON out)       |
| `src/engine.rs`  | `Engine` — NAPI wrapper around QuickJS runtime (`rquickjs::Runtime`)|
| `src/context.rs` | `Context` — NAPI wrapper around QuickJS context (`rquickjs::Context`)|
| `src/loader.rs`  | `Resolver` and `Loader` — QuickJS module resolution and loading     |
| `src/main.ts`    | vite-plus pack entry; re-exports the native bindings                |
| `src/*.test.ts`  | Unit tests                                                          |
| `build.ts`       | Drives `@napi-rs/cli` to build the `.node` addon into `dist/`       |
| `vite.config.ts` | `pack` / `test` / `lint` / `fmt` configuration                      |
| `Cargo.toml`     | Rust crate `alloy-engine-native` (`cdylib`)                         |

## NAPI surface

napi-rs exposes methods under their Rust (`snake_case`) name unless a `js_name`
override is set.

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

Wraps the QuickJS runtime (`rquickjs::Runtime`).

```ts
import { Engine, Resolver, Loader } from "@alloy-ts/engine";

const engine = new Engine();
engine.set_info("my-engine");
engine.set_memory_limit(1024 * 1024);
engine.set_max_stack_size(256 * 1024);
engine.set_gc_threshold(64 * 1024);
engine.run_gc();

const resolver = new Resolver();
resolver.with_file_resolver();
const loader = new Loader();
loader.with_script_loader();

engine.set_loader(resolver, loader);
```

### `Context`

Wraps QuickJS execution context (`rquickjs::Context`).

```ts
import { Context } from "@alloy-ts/engine";

const ctx = new Context();
const result = ctx.eval("1 + 1"); // "2"
const intResult = ctx.eval_int("2 + 3"); // 5
const boolResult = ctx.eval_bool("1 === 1"); // true
```

### `Resolver` and `Loader`

Module resolver and loader builders for QuickJS.

```ts
import { Resolver, Loader } from "@alloy-ts/engine";

const resolver = new Resolver();
resolver.add_path("./modules");

const loader = new Loader();
loader.with_script_loader();
loader.with_builtin_loader();
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

- Build the library (native addon + bundle):

```bash
npm run build
```

- Build only the native addon:

```bash
npm run build:napi       # debug
npm run build:release    # release
```

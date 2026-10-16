# JS Engine

A JavaScript execution engine with a native (Rust) core as a Node-API addon. The
Rust side is compiled to `alloy-engine.<platform>.node` via
[`@napi-rs/cli`](https://napi.rs), and the TypeScript side is bundled by
[vite-plus](https://viteplus.dev).

The addon exposes high-level bindings for JavaScript parsing, compilation, context management, and QuickJS runtime execution: `add`, `Parser`, `Compiler`, `Engine`, `Context`, `Loader`, and `Resolver`.

## Layout

| Path               | Role                                                                      |
| ------------------ | ------------------------------------------------------------------------- |
| `src/lib.rs`       | Crate root: module declarations and the `add` binding                     |
| `src/engine.rs`    | `Engine` — unified JS class wrapping Node-API context & QuickJS runtime   |
| `src/parser.rs`    | `Parser` — NAPI wrapper around `oxc_parser` (ESTree JSON out)             |
| `src/compiler.rs`  | `Compiler` — NAPI wrapper around oxc compiler/transformer                 |
| `src/context.rs`   | `Context` — NAPI wrapper around QuickJS execution context                 |
| `src/loader.rs`    | `Loader` & `Resolver` — QuickJS module resolution and loading bindings    |
| `src/main.ts`      | vite-plus pack entry; re-exports all native bindings                      |
| `src/*.test.ts`    | Unit tests (run by `vp test`)                                             |
| `docs/`            | Documentation guides (e.g. `docs/error-handling.md`)                      |
| `build.ts`         | Drives `@napi-rs/cli` to build the `.node` addon into `dist/`             |
| `vite.config.ts`   | `pack` / `test` / `lint` / `fmt` configuration                            |
| `Cargo.toml`       | Rust crate `alloy-engine-native` (`cdylib`)                               |

## NAPI surface

### `add(left: i32, right: i32) -> i32`

Plain function binding from `src/lib.rs`.

```ts
import { add } from "@alloy-ts/engine";

add(2, 3); // 5
```

### `Engine`

`Engine` is a unified JavaScript class that wraps both the Node-API context (`napi::Env`) and an embedded QuickJS runtime (`rquickjs::Runtime`).

Constructing `new Engine()` injects the Node-API environment and initializes the QuickJS runtime instance:

```ts
import { Engine } from "@alloy-ts/engine";

const engine = new Engine();

// Node-API context methods
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
engine.is_job_pending(); // false
engine.execPendingJob(); // false (or engine.execute_pending_job())
engine.run_gc(); // triggers garbage collection

// Error throwing helpers
engine.throw_error("custom error message", "ERR_CUSTOM");
engine.throw_range_error("out of bounds", "ERR_RANGE");
engine.throw_type_error("invalid type", "ERR_TYPE");
```

### `Context`

Wraps QuickJS execution contexts (`rquickjs::Context`). Allows evaluating scripts within an isolated QuickJS context:

```ts
import { Context } from "@alloy-ts/engine";

const ctx = new Context();
const result = ctx.eval("1 + 2"); // "3"
ctx.run_gc();
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

### `Compiler`

Wraps the oxc compiler. Execute compilation or transformation on source text:

```ts
import { Compiler } from "@alloy-ts/engine";

const compiler = new Compiler();
const output = compiler.execute("const x: number = 1;", "ts", "app.ts");
```

### `Loader` & `Resolver`

Module loading and path resolution helpers:

```ts
import { Resolver } from "@alloy-ts/engine";

const resolver = new Resolver();
const path = resolver.resolve("/app/src/index.js", "./utils.js"); // "/app/src/utils.js"
```

## Development

- Install dependencies:

```bash
npm install
```

- Build native NAPI module:

```bash
npm run build:napi
```

- Run unit & integration tests:

```bash
npm test
```

- Build release library:

```bash
npm run build
```

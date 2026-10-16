# JS Engine

A JavaScript execution engine with a native (Rust) core as a Node-API addon. The
Rust side is compiled to `alloy-engine.<platform>.node` via
[`@napi-rs/cli`](https://napi.rs), and the TypeScript side is bundled by
[vite-plus](https://viteplus.dev).

The addon exposes native bindings for `add`, `Parser` (oxc-backed), `Engine` (QuickJS runtime & N-API context), `Context` (QuickJS execution context), `Global`, and module `Loader`/`Resolver` types.

## Layout

| Path             | Role                                                                |
| ---------------- | ------------------------------------------------------------------- |
| `src/lib.rs`     | Crate root: module declarations and native exports                  |
| `src/context.rs` | `Context` — NAPI bindings for QuickJS execution contexts (`rquickjs`)|
| `src/engine.rs`  | `Engine` — JS class wrapping N-API context and QuickJS runtime      |
| `src/global.rs`  | `Global` — JS-visible handle for the global object                  |
| `src/loader.rs`  | `Loader` / `Resolver` — Module loader bindings                      |
| `src/parser.rs`  | `Parser` — NAPI wrapper around `oxc_parser` (ESTree JSON out)       |
| `src/main.ts`    | vite-plus pack entry; re-exports the native bindings                |
| `src/*.test.ts`  | Unit tests (run by `vp test`)                                       |
| `build.ts`       | Drives `@napi-rs/cli` to build the `.node` addon into `dist/`       |
| `vite.config.ts` | `pack` / `test` / `lint` / `fmt` configuration                      |
| `Cargo.toml`     | Rust crate `alloy-engine-native` (`cdylib`)                         |

## NAPI surface

### `Engine`

A JS class that owns both the N-API context handle (`napi::Env`) and a QuickJS runtime engine (`rquickjs::Runtime`).

```ts
import { Engine } from "@alloy-ts/engine";

const engine = new Engine();

// N-API context helpers
engine.create_string("hello"); // "hello"
engine.create_int32(42); // 42
engine.create_array(3); // [ <3 empty items> ]
engine.has_global("Math"); // true
engine.get_napi_version(); // host N-API version

// QuickJS Runtime Engine control
engine.execPendingJob(); // false
engine.memoryUsage(); // memory statistics
engine.setMemoryLimit(1024 * 1024);
engine.setMaxStackSize(512 * 1024);
engine.setGcThreshold(64 * 1024);
engine.runGc();
engine.setBuiltinLoader();
engine.setFileLoader();

engine.is_disposed(); // false while the addon is loaded
```

### `Context`

An execution context with its own global variables and stack, powered by QuickJS.

```ts
import { Context } from "@alloy-ts/engine";

const ctx = new Context();
ctx.eval("1 + 1"); // 2
ctx.jsonParse('{"a": 123}'); // { a: 123 }
ctx.jsonStringify({ b: "hello" }); // '{"b":"hello"}'
ctx.hasException(); // false
ctx.runGc();
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

### `add(left: i32, right: i32) -> i32`

Plain function binding from `src/lib.rs`.

```ts
import { add } from "@alloy-ts/engine";

add(2, 3); // 5
```

## Development

- Install dependencies:

```bash
npm install
```

- Run the unit tests:

```bash
npm test
```

- Build only the native addon:

```bash
npm run build:napi       # debug
npm run build:release    # release
```

- Build the library (native addon + bundle):

```bash
npm run build
```

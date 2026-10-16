# JS Engine

A JavaScript execution engine with a native (Rust) core as a Node-API addon. The
Rust side is compiled to `alloy-engine.<platform>.node` via
[`@napi-rs/cli`](https://napi.rs), and the TypeScript side is bundled by
[vite-plus](https://viteplus.dev).

The addon exposes native bindings: `add`, `Parser` (oxc-backed), `Engine` (QuickJS & N-API engine context), `Context`, `Global`, `Loader`, and `Resolver`.

## Layout

| Path             | Role                                                            |
| ---------------- | --------------------------------------------------------------- |
| `src/lib.rs`     | Crate root: module declarations and the `add` binding           |
| `src/context.rs` | `Context` — JS execution context bindings                       |
| `src/engine.rs`  | `Engine` — JS class wrapping N-API context & QuickJS engine API |
| `src/global.rs`  | `Global` — JS global object (`globalThis`) handle               |
| `src/loader.rs`  | `Loader` & `Resolver` — module loader and resolver interfaces   |
| `src/parser.rs`  | `Parser` — NAPI wrapper around `oxc_parser` (ESTree JSON out)   |
| `src/main.ts`    | vite-plus pack entry; re-exports the native bindings            |
| `src/*.test.ts`  | Unit tests (run by `vp test`)                                   |
| `build.ts`       | Drives `@napi-rs/cli` to build the `.node` addon into `dist/`   |
| `vite.config.ts` | `pack` / `test` / `lint` / `fmt` configuration                  |
| `Cargo.toml`     | Rust crate `alloy-engine-native` (`cdylib`)                     |

## NAPI surface

NAPI-RS converts Rust method names to camelCase for JavaScript according to standard JS conventions.

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

A JS class that owns the N-API context handle and QuickJS engine runtime options. Construct it with no arguments:

```ts
import { Engine, Loader, Resolver } from "@alloy-ts/engine";

const engine = new Engine();
engine.createString("hello"); // "hello"
engine.createInt32(42); // 42
engine.createArray(3); // [ <3 empty items> ]
engine.hasGlobal("Math"); // true
engine.getNapiVersion(); // host N-API version

engine.setInstanceData("vm-state");
engine.getInstanceData(); // "vm-state"

engine.setInfo("my-engine");
engine.setMemoryLimit(1024 * 1024);
engine.setMaxStackSize(256 * 1024);
engine.setGcThreshold(1024);

const resolver = new Resolver();
const loader = new Loader();
engine.setLoader(resolver, loader);

engine.runGc();
engine.isJobPending(); // false
engine.executePendingJob(); // false

engine.isDisposed(); // false while the addon is loaded
```

Available methods: `isDisposed`, `createArray`, `createString`,
`createInt32`, `createDouble`, `hasGlobal`, `getNapiVersion`,
`makeObject`, `setInstanceData`, `getInstanceData`, `setInfo`, `setMemoryLimit`,
`setMaxStackSize`, `setGcThreshold`, `setLoader`, `setDumpFlags`, `runGc`, `isJobPending`, `executePendingJob`.

### `Context`

Execution context for JavaScript code:

```ts
import { Context } from "@alloy-ts/engine";

const ctx = new Context();
ctx.eval("1 + 1"); // 2
ctx.compile("const a = 1;"); // true
ctx.jsonParse('{"hello":"world"}'); // { hello: "world" }
ctx.jsonStringify({ hello: "world" }); // '{"hello":"world"}'
```

Constructors & Factory methods: `new Context()`, `Context.base()`, `Context.full()`.
Available methods: `eval`, `compile`, `evalPromise`, `jsonParse`, `jsonStringify`, `hasException`, `runGc`.

### `Loader` & `Resolver`

Interfaces for module resolution and loading:

```ts
import { Loader, Resolver } from "@alloy-ts/engine";

const resolver = new Resolver();
resolver.resolve("src/app.js", "./utils.js"); // "src/utils.js"

const loader = new Loader();
loader.load("./utils.js");
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

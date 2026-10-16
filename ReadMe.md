# JS Engine

A JavaScript execution engine with a native (Rust) core as a Node-API addon. The
Rust side is compiled to `alloy-engine.<platform>.node` via
[`@napi-rs/cli`](https://napi.rs), and the TypeScript side is bundled by
[vite-plus](https://viteplus.dev).

The addon exposes native bindings: `add`, `Parser` (oxc-backed), `Engine` (a JS class wrapping N-API context and QuickJS runtime), `Context`, and `Loader`.

## Layout

| Path             | Role                                                            |
| ---------------- | --------------------------------------------------------------- |
| `src/lib.rs`     | Crate root: module declarations and the `add` binding           |
| `src/parser.rs`  | `Parser` — NAPI wrapper around `oxc_parser` (ESTree JSON out)   |
| `src/engine.rs`  | `Engine` — JS class wrapping N-API context and QuickJS runtime |
| `src/context.rs` | `Context` — QuickJS context wrapper for evaluating JavaScript  |
| `src/loader.rs`  | `Loader` — QuickJS module resolver and loader                   |
| `src/main.ts`    | vite-plus pack entry; re-exports the native bindings            |
| `src/*.test.ts`  | Unit tests (run by `vp test`)                                   |
| `build.ts`       | Drives `@napi-rs/cli` to build the `.node` addon into `dist/`   |
| `vite.config.ts` | `pack` / `test` / `lint` / `fmt` configuration                  |
| `Cargo.toml`     | Rust crate `alloy-engine-native` (`cdylib`)                     |

## NAPI surface

napi-rs automatically projects Rust `snake_case` method names to `camelCase` for JavaScript consumers.

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
`allow_return_outside_function`, `preserve_parens`, `allow_v8_intrinsics`.

### `Engine`

A JS class that owns the N-API context handle and QuickJS runtime instance. JS constructs it with an optional `Loader`:

```ts
import { Engine } from "@alloy-ts/engine";

const engine = new Engine();
engine.createString("hello"); // "hello"
engine.createInt32(42); // 42
engine.createArray(3); // [ <3 empty items> ]
engine.hasGlobal("Math"); // true
engine.getNapiVersion(); // host N-API version

engine.setInstanceData("vm-state");
engine.getInstanceData(); // "vm-state"

engine.isJobPending(); // false
engine.executePendingJob(); // false
engine.runGc();

engine.isDisposed(); // false while the addon is loaded
```

Available methods: `isDisposed`, `createArray`, `createString`,
`createInt32`, `createDouble`, `hasGlobal`, `getNapiVersion`,
`makeObject`, `setInstanceData`, `getInstanceData`, `setMemoryLimit`, `setMaxStackSize`, `runGc`, `isJobPending`, `executePendingJob`.

### `Context`

Wraps QuickJS `Context`.

```ts
import { Context, Engine } from "@alloy-ts/engine";

const engine = new Engine();
const context = Context.full(engine);
const result = context.eval("1 + 2"); // "3"
```

### `Loader`

Wraps QuickJS module resolver and loader.

```ts
import { Loader } from "@alloy-ts/engine";

const loader = new Loader();
loader.addModule("my_mod", "export const x = 42;");
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
npx vp test
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

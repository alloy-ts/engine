# JS Engine

A JavaScript execution engine with a native (Rust) core as a Node-API addon. The
Rust side is compiled to `alloy-engine.<platform>.node` via
[`@napi-rs/cli`](https://napi.rs), and the TypeScript side is bundled by
[vite-plus](https://viteplus.dev).

Today the addon exposes bindings: `add`, `Parser` (oxc-backed), `Compiler`, `Context`, `Engine` (wrapping both N-API context and QuickJS runtime), and module `Loader`/`Resolver` tools.

## Layout

| Path             | Role                                                            |
| ---------------- | --------------------------------------------------------------- |
| `src/lib.rs`     | Crate root: module declarations and the `add` binding           |
| `src/compiler.rs`| `Compiler` — NAPI wrapper around oxc AST parser/compiler        |
| `src/context.rs` | `Context` — QuickJS context NAPI wrapper                        |
| `src/engine.rs`  | `Engine` — JS class wrapping N-API context & QuickJS runtime    |
| `src/loader.rs`   | QuickJS `Loader` / `Resolver` NAPI bindings                     |
| `src/parser.rs`  | `Parser` — NAPI wrapper around `oxc_parser` (ESTree JSON out)   |
| `src/main.ts`    | vite-plus pack entry; re-exports the native bindings            |
| `src/*.test.ts`  | Unit tests (run by `vp test`)                                   |
| `build.ts`       | Drives `@napi-rs/cli` to build the `.node` addon into `dist/`   |
| `vite.config.ts` | `pack` / `test` / `lint` / `fmt` configuration                  |
| `Cargo.toml`     | Rust crate `alloy-engine-native` (`cdylib`)                     |

## NAPI surface

napi-rs converts Rust `snake_case` method names to JS `camelCase` conventions automatically.

### `add(left: i32, right: i32) -> i32`

Plain function binding from `src/lib.rs`.

```ts
import { add } from "@alloy-ts/engine";

add(2, 3); // 5
```

### `Parser` & `Compiler`

`Parser` wraps `oxc_parser`. Construct it with source text and an optional file path or bare extension used to infer the language / module kind.

```ts
import { Parser, Compiler } from "@alloy-ts/engine";

const parser = new Parser("const x = 1;", "src/app.tsx");
const { program, errors, fatalError, isFlowLanguage } = parser.parser();

const compiler = new Compiler();
const astJson = compiler.execute("const a = 42;", "ts", "app.ts");
```

### `Context`

`Context` exposes QuickJS execution contexts:

```ts
import { Context } from "@alloy-ts/engine";

const ctx = Context.full();
const result = ctx.eval("1 + 2 * 3"); // "7"
```

### `Engine`

A JS class that owns the N-API context handle and QuickJS runtime.

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

engine.setMemoryLimit(1024 * 1024);
engine.setMaxStackSize(256 * 1024);
engine.setGcThreshold(64 * 1024);
engine.runGc();
engine.isJobPending(); // false
engine.execPendingJob(); // false
engine.isDisposed(); // false while the addon is loaded
```

Available methods: `isDisposed`, `createArray`, `createString`, `createInt32`, `createDouble`, `hasGlobal`, `getNapiVersion`, `makeObject`, `setInstanceData`, `getInstanceData`, `setMemoryLimit`, `setMaxStackSize`, `setGcThreshold`, `runGc`, `isJobPending`, `executePendingJob`, `execPendingJob`.

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

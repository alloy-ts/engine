# JS Engine

A JavaScript execution engine with a native (Rust) core as a Node-API addon. The
Rust side is compiled to `alloy-engine.<platform>.node` via
[`@napi-rs/cli`](https://napi.rs), and the TypeScript side is bundled by
[vite-plus](https://viteplus.dev).

Today the addon exposes three bindings: `add`, `Parser` (oxc-backed) and
`Engine` (a JS class wrapping the N-API context).

## Layout

| Path             | Role                                                            |
| ---------------- | --------------------------------------------------------------- |
| `src/lib.rs`     | Crate root: module declarations and the `add` binding           |
| `src/parser.rs`  | `Parser` — NAPI wrapper around `oxc_parser` (ESTree JSON out)   |
| `src/engine.rs`  | `Engine` — JS class wrapping the N-API context (`napi::Engine`) |
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

A JS class that owns the N-API context handle. JS constructs it with no
arguments — NAPI injects the context:

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

engine.isDisposed(); // false while the addon is loaded
```

Available methods: `isDisposed`, `createArray`, `createString`,
`createInt32`, `createDouble`, `hasGlobal`, `getNapiVersion`,
`makeObject`, `setInstanceData`, `getInstanceData`, `setInfo`,
`setMemoryLimit`, `setMaxStackSize`, `setGcThreshold`, `setDumpFlags`,
`runGc`, `isJobPending`, `executePendingJob`, `setLoader`.

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
   public method refuses to use a dangling handle. `setInstanceData`'s
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

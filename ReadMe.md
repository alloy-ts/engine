# Engine

A JavaScript execution engine using [QuickJS](https://bellard.org/quickjs/)
native (Rust) [rquickjs](https://crates.io/crates/rquickjs) crate, shipped as
native binary via Node-API addon via [`@napi-rs/cli`](https://napi.rs).

This native addon exposes a single `Engine` class that is an **embedded QuickJS
engine** — a complete, isolated JS runtime courtesy of the
[`rquickjs`](https://crates.io/crates/rquickjs) safe high-level bindings to
[QuickJS](https://bellard.org/quickjs/) (runtime + contexts, module loading, GC,
job draining, …);

## Layout

| Path             | Role                                                              |
| ---------------- | ----------------------------------------------------------------- |
| `src/lib.rs`     | Crate root: module declarations and the `add` binding             |
| `src/parser.rs`  | `Parser` — NAPI wrapper around `oxc_parser` (ESTree JSON out)     |
| `src/engine.rs`  | `Engine` / `WeakEngine` — embedded QuickJS **and** host N-API env |
| `src/context.rs` | `Context` — a single embedded QuickJS realm                       |
| `src/loader.rs`  | `Loader` — host-provided QuickJS module loader/resolver           |
| `src/global.rs`  | `Global` — host N-API environment introspection helpers           |
| `src/main.ts`    | vite-plus pack entry; re-exports the native bindings              |
| `src/*.test.ts`  | Unit tests (run by `vp test`)                                     |
| `build.ts`       | Drives `@napi-rs/cli` to build the `.node` addon into `dist/`     |
| `vite.config.ts` | `pack` / `test` / `lint` / `fmt` configuration                    |
| `Cargo.toml`     | Rust crate `alloy-engine-native` (`cdylib`)                       |

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

A single JS class that represents the embedded QuickJS engine
(`rquickjs::Runtime`) captured at construction. JS constructs it with an
optional `Loader`:

```ts
import { Engine } from "@alloy-ts/engine";

const engine = new Engine(); // or: new Engine(loader)
```

The engine mints isolated QuickJS realms (`Context`s), runs module loaders, and
exposes runtime tuning and job draining. This is the `rquickjs` high-level
binding surface — the safe high-level bindings to
[QuickJS](https://bellard.org/quickjs/), heavily inspired by
[rlua](https://crates.io/crates/rlua).

`rquickjs` exposes the full QuickJS toolkit: a module system with `import` /
`export` (backed here by the `Loader`), multiple isolated contexts/realms,
host-defined `Class`es, `Promise`s, `TypedArray`s, `Proxy`es, explicit
garbage-collection control, and fine-grained memory/stack tuning. `Engine`
exposes a curated, N-API-friendly subset of that surface (the methods listed
below).

```ts
import { Engine, Loader } from "@alloy-ts/engine";

const loader = new Loader();
loader.add("math", "export const add = (a, b) => a + b;");

const engine = new Engine(loader);
const ctx = new Context();
ctx.evalModule('import { add } from "math"; globalThis.sum = add(2, 3);');
ctx.eval("globalThis.sum"); // 5

// Tuning & introspection:
engine.setMemoryLimit(1 << 20);
engine.setMaxStackSize(256 * 1024);
engine.setGcThreshold(10);
engine.setInfo("demo");
engine.memoryUsage(); // runtime memory stats
engine.runGc(); // force a garbage-collection sweep

// Promise / microtask jobs queue on the engine; drain them explicitly:
if (engine.is_job_pending()) {
  while (engine.execute_pending_job()) { /* run queued jobs */ }
}

// A weak handle that does NOT keep the engine alive:
const weak = engine.weak(); // -> WeakEngine
const strong = weak.try_ref(); // -> Engine | null (null once freed)
```

### `Context`

An embedded QuickJS realm. Either created standalone (`new Context()` — it spins
up its own private engine) or via `engine.create_context()`. Evaluate code with
`eval` / `eval_module` and enumerate globals with `global_keys`.

```ts
import { Context } from "@alloy-ts/engine";

const ctx = new Context();
ctx.eval("1 + 1"); // 2
ctx.eval_module("export const x = 5;"); // { x: 5 }
ctx.global_keys(); // string[]
```

`Context` methods: `eval`, `eval_module`, `global_keys`, `run_gc`,
`execute_pending_job`.

### `Loader`

A host-provided QuickJS module loader/resolver: register module source text and
hand the `Loader` to `new Engine(loader)` (or `engine.set_loader(loader)`).
Modules become importable from any context created by that engine.

### `Global`

Host N-API environment introspection helpers for the host global object.

```ts
import { Global } from "@alloy-ts/engine";

const g = new Global();
g.has("Math"); // true
g.stringify({ a: 1 }); // '{"a":1}'
const id = g.setTimeout(() => {}, 10);
g.clearTimeout(id);
```

`Global` methods: `setTimeout`, `setInterval`, `clearTimeout`, `clearInterval`,
`stringify`, `has`.

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

- `eval_module` returns the module **namespace** serialized to JSON, so exported
  _functions_ collapse to `null` (they are not JSON-representable). Exported
  data (`const`/`let` values, arrays, objects) is preserved. If you need to call
  an exported function, evaluate an expression in the context that invokes it.
- The `Loader` is host-driven (pre-registered source map) rather than a
  per-import JS callback, because rquickjs resolves/loads synchronously inside
  evaluation (see `Loader` above).

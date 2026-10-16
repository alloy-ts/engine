# Engine

A JavaScript execution engine using [QuickJS](https://bellard.org/quickjs/) native (Rust) [rquickjs](https://crates.io/crates/rquickjs) crate, shipped as native binary via
Node-API addon via [`@napi-rs/cli`](https://napi.rs), and the TypeScript side is bundled by
[vite-plus](https://viteplus.dev).

The addon exposes a single `Engine` class that is **two things at once**:

- an **embedded QuickJS engine** — a complete, isolated JS runtime courtesy of
  the [`rquickjs`](https://crates.io/crates/rquickjs) safe high-level bindings
  to [QuickJS](https://bellard.org/quickjs/) (runtime + contexts, module
  loading, GC, job draining, …); and
- a view of the **host N-API environment** (Node's V8) through the `napi::Env`
  that NAPI injects at construction — so JS can build/inspect host values and
  persist host-level state on the same object.

Both facets are methods on the same `Engine`. There is one constructor,
`new Engine(loader?)`, which simultaneously captures the host `Env` and creates
the embedded QuickJS runtime (optionally seeded with a `Loader`).

The other bindings are `add` (a plain function), `Compiler` / `Parser` (oxc-backed), and the
companion classes `Context`, `WeakEngine`, `Loader`, and `Global`.

Detailed API documentation is available in the [`docs/`](./docs) directory:

- [`docs/engine.md`](./docs/engine.md) — Embedded QuickJS Runtime & Host N-API Engine API
- [`docs/context.md`](./docs/context.md) — Isolated QuickJS Context & Realm API
- [`docs/loader.md`](./docs/loader.md) — ES Module Loader API
- [`docs/compiler.md`](./docs/compiler.md) — OXC Compiler AST Parser API

## Layout

| Path              | Role                                                               |
| ----------------- | ------------------------------------------------------------------ |
| `src/lib.rs`      | Crate root: module declarations and the `add` binding              |
| `src/compiler.rs` | `Compiler` — NAPI wrapper around `oxc_parser` / `oxc_estree`       |
| `src/parser.rs`   | `Parser` — NAPI wrapper around `oxc_parser` (ESTree JSON out)      |
| `src/engine.rs`   | `Engine` / `WeakEngine` — embedded QuickJS **and** host N-API env  |
| `src/context.rs`  | `Context` — a single embedded QuickJS realm                        |
| `src/loader.rs`   | `Loader` — host-provided QuickJS module loader/resolver            |
| `src/global.rs`   | `Global` — host N-API environment introspection helpers            |
| `src/main.ts`     | vite-plus pack entry; re-exports the native bindings               |
| `src/*.test.ts`   | Unit tests (run by `vp test`)                                      |
| `docs/*.md`       | Documentation guides for `Engine`, `Context`, `Loader`, `Compiler` |
| `build.ts`        | Drives `@napi-rs/cli` to build the `.node` addon into `dist/`      |
| `vite.config.ts`  | `pack` / `test` / `lint` / `fmt` configuration                     |
| `Cargo.toml`      | Rust crate `alloy-engine-native` (`cdylib`)                        |

## NAPI surface

napi-rs exposes methods under their Rust (`snake_case`) name unless a `js_name`
override is set, so the JS API mirrors the Rust names below.

### `add(left: number, right: number) -> number`

Plain function binding from `src/lib.rs`.

```ts
import { add } from "@alloy-ts/engine";

add(2, 3); // 5
```

### `Compiler`

Wraps `oxc_parser` / `oxc_estree`. Execute compilation/transformation on source text:

```ts
import { Compiler } from "@alloy-ts/engine";

const compiler = new Compiler();
const astJson = compiler.execute("const x = 1;", "js", "index.js");
```

See [`docs/compiler.md`](./docs/compiler.md) for full documentation.

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

A single JS class that owns **both** the embedded QuickJS engine
(`rquickjs::Runtime`) and the host N-API environment (`napi::Env`) captured at
construction. JS constructs it with an optional `Loader`:

```ts
import { Engine } from "@alloy-ts/engine";

const engine = new Engine(); // or: new Engine(loader)
```

#### Facet A — embedded QuickJS runtime

The engine mints isolated QuickJS realms (`Context`s), runs module loaders, and
exposes runtime tuning and job draining. This is the `rquickjs` high-level
binding surface — the safe high-level bindings to
[QuickJS](https://bellard.org/quickjs/), heavily inspired by
[rlua](https://crates.io/crates/rlua).

```ts
import { Engine, Loader } from "@alloy-ts/engine";

const loader = new Loader();
loader.add("math", "export const add = (a, b) => a + b;");

const engine = new Engine(loader);
const ctx = engine.create_context();
ctx.eval_module('import { add } from "math"; globalThis.sum = add(2, 3);');
ctx.eval("globalThis.sum"); // 5

// Tuning & introspection:
engine.set_memory_limit(1 << 20);
engine.set_max_stack_size(256 * 1024);
engine.set_gc_threshold(10);
engine.set_info("demo");
engine.memory_usage(); // runtime memory stats
engine.run_gc(); // force a garbage-collection sweep

// Promise / microtask jobs queue on the engine; drain them explicitly:
if (engine.is_job_pending()) {
  while (engine.execPendingJob()) {
    /* run queued jobs */
  }
}

// A weak handle that does NOT keep the engine alive:
const weak = engine.weak(); // -> WeakEngine
const strong = weak.try_ref(); // -> Engine | null (null once freed)
```

Runtime methods: `set_loader`, `create_context`, `weak` (→ `WeakEngine`),
`set_memory_limit`, `set_max_stack_size`, `set_gc_threshold`, `set_info`,
`run_gc`, `memory_usage`, `is_job_pending`, `execute_pending_job` (also available as `execPendingJob` and `exec_pending_job`).

#### Facet B — host N-API environment

These methods operate on the host (`napi::Env`) captured at construction, so JS
can build and inspect values in the _host_ realm (Node's V8) and persist
host-level state on the Agent.

```ts
import { Engine } from "@alloy-ts/engine";

const engine = new Engine();
engine.create_string("hello"); // "hello"
engine.create_int32(42); // 42
engine.create_double(1.5); // 1.5
engine.create_array(3); // [ <3 empty items> ]
engine.has_global("Math"); // true (host global)
engine.get_napi_version(); // host N-API version, e.g. 9

const obj = engine.make_object(); // { ok: true, engine: "napi" }

engine.set_instance_data("vm-state");
engine.get_instance_data(); // "vm-state"

engine.is_disposed(); // false while the addon is loaded
```

Host methods: `is_disposed`, `create_array`, `create_string`, `create_int32`,
`create_double`, `has_global`, `get_napi_version`, `make_object`,
`set_instance_data`, `get_instance_data`.

### `Context`

An embedded QuickJS realm. Either created standalone (`new Context()` — it
spins up its own private engine) or via `engine.create_context()`.

See [`docs/context.md`](./docs/context.md) for full documentation.

### `Loader`

A host-provided QuickJS module loader/resolver: register module source text and
hand the `Loader` to `new Engine(loader)` (or `engine.set_loader(loader)`).

See [`docs/loader.md`](./docs/loader.md) for full documentation.

### `Global`

Host N-API environment introspection helpers for the host global object.

## Development

- Install dependencies:

```bash
vp install
```

- Run the unit tests:

```bash
vp test
```

- Build the library (native addon + bundle):

```bash
npm run build
```

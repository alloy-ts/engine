# Engine

A JavaScript execution engine with a native (Rust) core, shipped as a
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

The other bindings are `add` (a plain function), `Parser` and `Compiler` (oxc-backed), and the
companion classes `Context`, `WeakEngine`, `Loader`, and `Global`.

## Layout

| Path             | Role                                                                 |
| ---------------- | -------------------------------------------------------------------- |
| `src/lib.rs`     | Crate root: module declarations and the `add` binding                 |
| `src/parser.rs`  | `Parser` — NAPI wrapper around `oxc_parser` (ESTree JSON out)         |
| `src/compiler.rs`| `Compiler` — oxc compiler wrapper with compact ESTree serialization  |
| `src/engine.rs`  | `Engine` / `WeakEngine` — embedded QuickJS **and** host N-API env     |
| `src/context.rs` | `Context` — a single embedded QuickJS realm                            |
| `src/loader.rs`  | `Loader` — host-provided QuickJS module loader/resolver               |
| `src/global.rs`  | `Global` — host N-API environment introspection helpers               |
| `src/main.ts`    | vite-plus pack entry; re-exports the native bindings                  |
| `src/*.test.ts`  | Unit tests (run by `vp test`)                                        |
| `build.ts`       | Drives `@napi-rs/cli` to build the `.node` addon into `dist/`         |
| `vite.config.ts` | `pack` / `test` / `lint` / `fmt` configuration                       |
| `Cargo.toml`     | Rust crate `alloy-engine-native` (`cdylib`)                          |

## NAPI surface

napi-rs exposes methods under both camelCase (standard JS style, e.g. `execPendingJob`) and `snake_case` aliases.

### `add(left: number, right: number) -> number`

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

### `Compiler`

Wraps the oxc parser and produces compact serialized ESTree output.

```ts
import { Compiler } from "@alloy-ts/engine";

const compiler = new Compiler();
const estreeJson = compiler.execute("const a = 1;", "js", "index.js");
```

### `Engine`

A single JS class that owns **both** the embedded QuickJS engine
(`rquickjs::Runtime`) and the host N-API environment (`napi::Env`) captured at
construction. JS constructs it with an optional `Loader`:

```ts
import { Engine } from "@alloy-ts/engine";

const engine = new Engine();      // or: new Engine(loader)
```

#### Facet A — embedded QuickJS runtime

The engine mints isolated QuickJS realms (`Context`s), runs module loaders, and
exposes runtime tuning and job draining. Both camelCase (`execPendingJob`, `isJobPending`, `createContext`, `runGc`) and snake_case (`execute_pending_job`, `is_job_pending`, `create_context`, `run_gc`) methods are supported.

```ts
import { Engine, Loader } from "@alloy-ts/engine";

const loader = new Loader();
loader.add("math", "export const add = (a, b) => a + b;");

const engine = new Engine(loader);
const ctx = engine.createContext(); // or engine.create_context()
ctx.eval_module('import { add } from "math"; globalThis.sum = add(2, 3);');
ctx.eval("globalThis.sum"); // 5

// Tuning & introspection:
engine.setMemoryLimit(1 << 20);
engine.setMaxStackSize(256 * 1024);
engine.setGcThreshold(10);
engine.setInfo("demo");
engine.memoryUsage();            // runtime memory stats
engine.runGc();                  // force a garbage-collection sweep

// Promise / microtask jobs queue on the engine; drain them explicitly:
if (engine.isJobPending()) {
  while (engine.execPendingJob()) { /* run queued jobs */ }
}

// A weak handle that does NOT keep the engine alive:
const weak = engine.weak();       // -> WeakEngine
const strong = weak.try_ref();    // -> Engine | null (null once freed)
```

Runtime methods: `setLoader` / `set_loader`, `createContext` / `create_context`, `weak` (→ `WeakEngine`),
`setMemoryLimit` / `set_memory_limit`, `setMaxStackSize` / `set_max_stack_size`, `setGcThreshold` / `set_gc_threshold`, `setInfo` / `set_info`,
`runGc` / `run_gc`, `memoryUsage` / `memory_usage`, `isJobPending` / `is_job_pending`, `execPendingJob` / `executePendingJob` / `execute_pending_job`.

#### Facet B — host N-API environment

These methods operate on the host (`napi::Env`) captured at construction. Both camelCase and snake_case aliases are available.

```ts
import { Engine } from "@alloy-ts/engine";

const engine = new Engine();
engine.createString("hello");  // "hello"
engine.createInt32(42);        // 42
engine.createDouble(1.5);      // 1.5
engine.createArray(3);         // [ <3 empty items> ]
engine.hasGlobal("Math");      // true (host global)
engine.getNapiVersion();      // host N-API version, e.g. 9

const obj = engine.makeObject(); // { ok: true, engine: "napi" }

engine.setInstanceData("vm-state");
engine.getInstanceData();     // "vm-state"

engine.isDisposed();          // false while the addon is loaded
```

### `Context`

An embedded QuickJS realm. Either created standalone (`new Context()`) or via `engine.createContext()`. Evaluate code with `eval` / `eval_module` and enumerate globals with `global_keys`.

### `Loader`

A host-provided QuickJS module loader/resolver: register module source text and
hand the `Loader` to `new Engine(loader)` (or `engine.setLoader(loader)`).

### `Global`

Host N-API environment introspection helpers for the host global object.

## Development

- Install dependencies:

```bash
npm install
```

- Run unit tests:

```bash
npx vp test
```

- Build native addon:

```bash
npm run build:napi
```

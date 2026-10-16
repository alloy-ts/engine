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

The other bindings are `add` (a plain function), `Parser` (oxc-backed), and the
companion classes `Context`, `WeakEngine`, `Loader`, and `Global`.

## Layout

| Path             | Role                                                                 |
| ---------------- | -------------------------------------------------------------------- |
| `src/lib.rs`     | Crate root: module declarations and the `add` binding                 |
| `src/parser.rs`  | `Parser` — NAPI wrapper around `oxc_parser` (ESTree JSON out)         |
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

Both idiomatic JS camelCase methods (e.g. `createContext`, `execPendingJob`, `executePendingJob`) and snake_case methods (e.g. `create_context`, `execute_pending_job`) are supported on the `Engine` API.

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

`parser()` collects recoverable diagnostics instead of throwing (check `errors`
/ `fatalError`); `parse_expression()` throws on a syntax error.

`ParseOptions` mirrors `oxc_parser::ParseOptions` — every field is optional and
falls back to oxc's default: `parse_regular_expression`,
`allow_return_outside_function`, `preserve_parens`, `allow_v8_intrinsics`,
`enable_ident_hashes`.

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
const ctx = engine.create_context();
ctx.eval_module('import { add } from "math"; globalThis.sum = add(2, 3);');
ctx.eval("globalThis.sum"); // 5

// Tuning & introspection:
engine.set_memory_limit(1 << 20);
engine.set_max_stack_size(256 * 1024);
engine.set_gc_threshold(10);
engine.set_info("demo");
engine.memory_usage();           // runtime memory stats
engine.run_gc();                 // force a garbage-collection sweep

// Promise / microtask jobs queue on the engine; drain them explicitly:
if (engine.is_job_pending()) {
  while (engine.execute_pending_job()) { /* run queued jobs */ }
}

// A weak handle that does NOT keep the engine alive:
const weak = engine.weak();       // -> WeakEngine
const strong = weak.try_ref();    // -> Engine | null (null once freed)
```

Runtime methods: `set_loader` / `setLoader`, `create_context` / `createContext`, `weak` (→ `WeakEngine`),
`set_memory_limit` / `setMemoryLimit`, `set_max_stack_size` / `setMaxStackSize`, `set_gc_threshold` / `setGcThreshold`, `set_info` / `setInfo`,
`run_gc` / `runGc`, `memory_usage` / `memoryUsage`, `is_job_pending` / `isJobPending`, `execute_pending_job` / `executePendingJob` / `execPendingJob`.

A few real QuickJS semantics worth knowing:

- Contexts created from one engine are **separate realms** — they share the
  engine, but each has its own `globalThis`. A global set on one context is not
  visible on another.
- A thrown **promise reaction** becomes an *unhandled rejection* (the job itself
  succeeds, so `execute_pending_job` does **not** throw). A **`queueMicrotask`**
  job that throws **does** propagate out of `execute_pending_job`.

#### Facet B — host N-API environment

These methods operate on the host (`napi::Env`) captured at construction, so JS
can build and inspect values in the *host* realm (Node's V8) and persist
host-level state on the Agent.

```ts
import { Engine } from "@alloy-ts/engine";

const engine = new Engine();
engine.create_string("hello"); // "hello"
engine.create_int32(42);       // 42
engine.create_double(1.5);     // 1.5
engine.create_array(3);        // [ <3 empty items> ]
engine.has_global("Math");     // true (host global)
engine.get_napi_version();     // host N-API version, e.g. 9

const obj = engine.make_object(); // { ok: true, engine: "napi" }

engine.set_instance_data("vm-state");
engine.get_instance_data();    // "vm-state"

engine.is_disposed();         // false while the addon is loaded
```

Host methods: `is_disposed` / `isDisposed`, `create_array` / `createArray`, `create_string` / `createString`, `create_int32` / `createInt32`,
`create_double` / `createDouble`, `has_global` / `hasGlobal`, `get_napi_version` / `getNapiVersion`, `make_object` / `makeObject`,
`set_instance_data` / `setInstanceData`, `get_instance_data` / `getInstanceData`.

#### Using both facets together

The two facets run in *separate* realms: the host `napi::Env` is Node's V8,
while `create_context()` spins up a QuickJS realm — so values are not shared by
reference. The bridge between them is **JSON marshalling**: `Context.eval` /
`eval_module` serialize their QuickJS result into a plain *host* value you can
use directly in Node.

```ts
import { Engine } from "@alloy-ts/engine";

const engine = new Engine();

// Facet A — compute inside QuickJS; the result comes back as a host value.
const ctx = engine.create_context();
const out = ctx.eval("Array.from({ length: 3 }, (_, i) => i * 2)"); // [0, 2, 4]
while (engine.execute_pending_job()) { /* drain any queued jobs */ }

// Facet B — reflect into the host environment and persist host-level state.
engine.set_instance_data(JSON.stringify(out)); // "[0,2,4]"
engine.has_global("Map");                      // true (host global)

// You can also build host values explicitly; they live in Node's V8, not QuickJS.
const hostArr = engine.create_array(3);  // [ <3 empty items> ]
const hostObj = engine.make_object();    // { ok: true, engine: "napi" }
```

#### The N-API context contract

`napi::Env` is the handle the N-API implementation uses to persist VM-specific
state. `Engine` follows its rules:

1. **The same environment is reused for nested calls.** Every host method does
   its N-API work through the `Env` captured at construction.
2. **It must not be shared between addon instances or Worker threads.** Each
   `Engine` captures its own environment; the fields are private so they cannot
   leak out.
3. **It becomes invalid when the addon instance is unloaded.** `Engine::new`
   registers an environment cleanup hook that flips a disposed flag, and every
   public method refuses to use a dangling handle. `set_instance_data`'s
   finalize callback observes the same unload event.

### `Context`

An embedded QuickJS realm. Either created standalone (`new Context()` — it
spins up its own private engine) or via `engine.create_context()`. Evaluate
code with `eval` / `eval_module` and enumerate globals with `global_keys`.

```ts
import { Context } from "@alloy-ts/engine";

const ctx = new Context();
ctx.eval("1 + 1");                  // 2
ctx.eval_module("export const x = 5;"); // { x: 5 }
ctx.global_keys();                 // string[]
```

`Context` methods: `eval`, `eval_module`, `global_keys`, `run_gc`,
`execute_pending_job`.

### `Loader`

A host-provided QuickJS module loader/resolver: register module source text and
hand the `Loader` to `new Engine(loader)` (or `engine.set_loader(loader)`).
Modules become importable from any context created by that engine.

```ts
import { Loader, Engine } from "@alloy-ts/engine";

const loader = new Loader();
loader.add("math", "export const add = (a, b) => a + b;");
loader.has("math");  // true
loader.keys();       // ["math"]

const engine = new Engine(loader);
const ctx = engine.create_context();
ctx.eval_module('import { add } from "math"; globalThis.sum = add(2, 3);');
```

`Loader` methods: `add`, `remove`, `has`, `keys`, `clear`.

> Note: rquickjs calls `Loader::load` *synchronously* mid-evaluation and needs a
> compiled module back inline, so the loader is host-driven (JS pre-registers
> sources) rather than a per-import JS callback.

### `Global`

Host N-API environment introspection helpers for the host global object.

```ts
import { Global } from "@alloy-ts/engine";

const g = new Global();
g.has("Math");              // true
g.stringify({ a: 1 });      // '{"a":1}'
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
  *functions* collapse to `null` (they are not JSON-representable). Exported
  data (`const`/`let` values, arrays, objects) is preserved. If you need to call
  an exported function, evaluate an expression in the context that invokes it.
- The `Loader` is host-driven (pre-registered source map) rather than a
  per-import JS callback, because rquickjs resolves/loads synchronously inside
  evaluation (see `Loader` above).

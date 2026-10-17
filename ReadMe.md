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

The other bindings are `add` (a plain function), `Parser` (oxc-backed), `Compiler` (oxc-backed AST compiler), and the
companion classes `Context`, `WeakEngine`, `Loader`, and `Global`.

## Layout

| Path             | Role                                                                 |
| ---------------- | -------------------------------------------------------------------- |
| `src/lib.rs`     | Crate root: module declarations and the `add` binding                 |
| `src/parser.rs`  | `Parser` — NAPI wrapper around `oxc_parser` (ESTree JSON out)         |
| `src/compiler.rs`| `Compiler` — NAPI wrapper around `oxc_parser` AST serialization       |
| `src/engine.rs`  | `Engine` / `WeakEngine` — embedded QuickJS **and** host N-API env     |
| `src/context.rs` | `Context` — a single embedded QuickJS realm                            |
| `src/loader.rs`  | `Loader` — host-provided QuickJS module loader/resolver               |
| `src/global.rs`  | `Global` — host N-API environment introspection helpers               |
| `src/main.ts`    | vite-plus pack entry; re-exports the native bindings                  |
| `src/*.test.ts`  | Unit tests (run by `vp test`)                                        |
| `docs/*.md`      | Detailed documentation for `Engine`, `Context`, and `Loader`          |
| `build.ts`       | Drives `@napi-rs/cli` to build the `.node` addon into `dist/`         |
| `vite.config.ts` | `pack` / `test` / `lint` / `fmt` configuration                       |
| `Cargo.toml`     | Rust crate `alloy-engine-native` (`cdylib`)                          |

## NAPI surface

Methods support both JavaScript standard `camelCase` and `snake_case` names.

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

Wraps `oxc_parser` to parse and serialize source code into ESTree AST JSON string output using oxc's `CompactSerializer`.

```ts
import { Compiler } from "@alloy-ts/engine";

const compiler = new Compiler();
const astJson = compiler.execute("const a = 10;", "js", "index.js");
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
exposes runtime tuning and job draining. This is the `rquickjs` high-level
binding surface — the safe high-level bindings to
[QuickJS](https://bellard.org/quickjs/), heavily inspired by
[rlua](https://crates.io/crates/rlua).

```ts
import { Engine, Loader } from "@alloy-ts/engine";

const loader = new Loader();
loader.add("math", "export const add = (a, b) => a + b;");

const engine = new Engine(loader);
const ctx = engine.createContext(); // or engine.create_context()
ctx.evalModule('import { add } from "math"; globalThis.sum = add(2, 3);');
ctx.eval("globalThis.sum"); // 5

// Tuning & introspection:
engine.setMemoryLimit(1 << 20);
engine.setMaxStackSize(256 * 1024);
engine.setGcThreshold(10);
engine.setInfo("demo");
engine.memoryUsage();           // runtime memory stats
engine.runGc();                 // force a garbage-collection sweep

// Promise / microtask jobs queue on the engine; drain them explicitly:
if (engine.isJobPending()) {
  while (engine.execPendingJob()) { /* run queued jobs */ }
}

// A weak handle that does NOT keep the engine alive:
const weak = engine.weak();       // -> WeakEngine
const strong = weak.tryRef();     // -> Engine | null (null once freed)
```

Runtime methods support both `camelCase` (`createContext`, `setLoader`, `setMemoryLimit`, `setMaxStackSize`, `setGcThreshold`, `setInfo`, `runGc`, `memoryUsage`, `isJobPending`, `execPendingJob`, `executePendingJob`) and `snake_case` (`create_context`, `set_loader`, `set_memory_limit`, etc.).

#### Facet B — host N-API environment

These methods operate on the host (`napi::Env`) captured at construction, so JS
can build and inspect values in the *host* realm (Node's V8) and persist
host-level state on the Agent.

```ts
import { Engine } from "@alloy-ts/engine";

const engine = new Engine();
engine.createString("hello"); // "hello"
engine.createInt32(42);       // 42
engine.createDouble(1.5);     // 1.5
engine.createArray(3);        // [ <3 empty items> ]
engine.hasGlobal("Math");     // true (host global)
engine.getNapiVersion();     // host N-API version, e.g. 9

const obj = engine.makeObject(); // { ok: true, engine: "napi" }

engine.setInstanceData("vm-state");
engine.getInstanceData();    // "vm-state"

engine.isDisposed();         // false while the addon is loaded
```

Host methods support both `camelCase` (`isDisposed`, `createArray`, `createString`, `createInt32`, `createDouble`, `hasGlobal`, `getNapiVersion`, `makeObject`, `setInstanceData`, `getInstanceData`) and `snake_case`.

### `Context`

An embedded QuickJS realm. See [`docs/context.md`](docs/context.md) for full documentation.

```ts
import { Context } from "@alloy-ts/engine";

const ctx = new Context();
ctx.eval("1 + 1");                  // 2
ctx.evalModule("export const x = 5;"); // { x: 5 }
ctx.globalKeys();                 // string[]
```

### `Loader`

A host-provided QuickJS module loader/resolver. See [`docs/loader.md`](docs/loader.md) for full documentation.

```ts
import { Loader, Engine } from "@alloy-ts/engine";

const loader = new Loader();
loader.add("math", "export const add = (a, b) => a + b;");
loader.has("math");  // true
loader.keys();       // ["math"]

const engine = new Engine(loader);
const ctx = engine.createContext();
ctx.evalModule('import { add } from "math"; globalThis.sum = add(2, 3);');
```

### `Global`

Host N-API environment introspection helpers for the host global object.

```ts
import { Global } from "@alloy-ts/engine";

const g = new Global();
g.has("Math");              // true
g.stringify({ a: 1 });      // '{"a":1}'
```

## Detailed Documentation

- [`docs/engine.md`](docs/engine.md) - Full API details for `Engine` and `WeakEngine`.
- [`docs/context.md`](docs/context.md) - Details for QuickJS `Context` execution.
- [`docs/loader.md`](docs/loader.md) - Details for module resolution with `Loader`.

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

- Build the library:

```bash
npm run build
```

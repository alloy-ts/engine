# Engine

## What is `Engine`?

The `Engine` is the **root object** of the boa N-API addon, which surfaces the
[`boa_engine`](https://docs.rs/boa_engine/0.22.0/boa_engine/) Rust crate that
implements ECMAScript: the standard-library builtins, a `Context` (realm +
global object + garbage-collected heap + virtual machine), a byte compiler, and
a VM. Its crate-root re-exports everything useful through a `prelude`:

```rust
boa_engine::{
    bigint::JsBigInt, context::Context, error::{...}, host_defined::HostDefined,
    interop::{IntoJsFunctionCopied, UnsafeIntoJsFunction}, module::{IntoJsModule, Module},
    native_function::NativeFunction, object::{JsData, JsObject, NativeObject},
    script::Script, string::{JsStr, JsString}, symbol::JsSymbol,
    value::{JsValue, JsVariant, js_object, js_value},
}
pub use boa_parser::Source;
```

The crate is organised into modules, each a different facet of the engine:

| Module                                                               | Purpose                                       | NAPI-relevant?                           |
| -------------------------------------------------------------------- | --------------------------------------------- | ---------------------------------------- |
| `context`                                                            | `Context` — the execution environment         | ✅ root of every JS call                 |
| `value`                                                              | `JsValue` / `JsVariant` — runtime values      | ❌ lives inside a `Context` (GC-managed) |
| `object`                                                             | `JsObject`, `NativeObject`, `JsData`          | ❌ internal                              |
| `symbol`                                                             | `JsSymbol` (well-known + global registry)     | ⚠️ partly (global registry)              |
| `string`                                                             | `JsString` / `JsStr` (interner)               | ❌ internal                              |
| `bigint`                                                             | `JsBigInt`                                    | ❌ internal                              |
| `builtins`                                                           | `Object`/`Array`/`Promise`/… standard library | ✅ reachable via a `Context`             |
| `native_function`                                                    | `NativeFunction`                              | ❌ Rust-only                             |
| `class`                                                              | native class registration                     | ❌ Rust-only                             |
| `module`                                                             | `Module` (ES modules)                         | ⚠️ parse only                            |
| `script`                                                             | `Script` (parse + eval)                       | ✅ `Script` class                        |
| `bytecompiler`                                                       | `Compiler` (bytecode)                         | ✅ `Compiler` class                      |
| `parser`                                                             | `Parser` (AST)                                | ✅ `Parser` class                        |
| `realm`                                                              | `Realm` (realm record)                        | ✅ `Realm` handle + free fns             |
| `error`                                                              | `JsError`, `JsNativeError`, `EngineError`     | ⚠️ surfaced as JS exceptions             |
| `vm` / `job` / `environments` / `property` / `interop` / `optimizer` | VM, jobs, scopes, etc.                        | ❌ internal                              |
| `prelude`                                                            | re-exports above                              | n/a                                      |

## Key constraint: there is no boa "Engine" object

Unlike QuickJS (`rquickjs::Runtime`), **boa has no central runtime/engine
handle**. The `Context` _is_ the engine: it owns the realm, the global object,
the interner, the clock, the job executor and the entire GC heap. Because:

- those types are `!Send` / `!Sync` and GC-managed, and
- JS values (`JsValue`/`JsObject`/`JsSymbol`) are meaningless outside the
  `Context` that allocated them,

…they **cannot** be napi fields or method return values directly. napi-rs
requires every exported class to be `Send + Sync`, so boa's `Context` lives in a
thread-local registry behind an opaque `u32` id (see `docs/api/context.md`).

**Consequence for the design**: an NAPI `Engine` cannot be a thin wrapper over a
boa type. It must be a **facade / root object** that exposes the
_realm-independent_ and _host-environment_ capabilities of boa, and that
_creates_ the `Context`s where actual JS values live.

### The `Engine` public API

`boa_engine` is the crate what ultimately `Engine`/`Context` wraps. Its crate
root re-exports a `prelude` of the most-used items; the rest live in modules.
The public surface a JS author can _reach_ (always through a `Context`,
`Script`, or `Realm`) is:

- **Execution** — `Context` (realm + global + GC heap + VM),
  `Context::default()`, `Context::builder()` (configurable: realm, clock, module
  loader, Intl/ICU, runtime limits, optimizer), `Context::eval` (returns the
  completion value and does **not** drain microtasks), `Context::compile` (to
  `Script`), `Context::parse`, `Context::run_jobs` (drain microtasks),
  `Context::register_global_callable` / `register_global_builtin_callable`,
  `Context::set_runtime_limits`, `Context::optimizer_options`,
  `Context::enter_realm` / `Context::realm`.
- **Source / Script / Module** — `Source` (`from_bytes`, `from_file`), `Script`
  (`Script::parse`, `Script::evaluate`), `Module` (`Module::parse`,
  `Module::evaluate`) for ES modules.
- **Values** — `JsValue`
  (`undefined`/`null`/`boolean`/`number`/`string`/`symbol`/ `object`/`bigint`),
  `JsObject`, `JsString`, `JsSymbol`, `JsBigInt`, and errors `JsError` /
  `JsNativeError` (surfaced as JS exceptions).
- **Native interop (Rust-only)** — `NativeFunction` (`from_fn`,
  `from_copy_closure`, `from_closure`), `JsObject`/`JsData`/`NativeObject`, the
  `js_str!` / `js_object!` macros, `TryIntoJsResult`,
  `JsArgs::get_or_undefined`.
- **Object model** — `object` module: `ObjectInitializer`, `FunctionBinding`,
  property descriptors, `PropertyKey`.
- **Builtins (reachable via a `Context`'s global)** — `Object`, `Array`,
  `ArrayBuffer`, `TypedArray`, `DataView`, `Map`, `Set`, `WeakMap`, `WeakSet`,
  `Promise`, `Proxy`, `Reflect`, `Symbol`, `JSON`, `Math`, `Date`, `RegExp`,
  `Error` (+ subtypes), `Function`, `Generator`, `AsyncFunction`, `Iterator`,
  `WeakRef`, `FinalizationRegistry`, and `Intl` (with the `intl` feature).
- **Realm** — `Realm` (`Realm::create`, `enter`, `exit`).
- **Optimizer** — `optimizer` module: `OptimizerOptions` (dead-code elimination,
  constant folding, …) applied at compile time.
- **Error types** — `EngineError`, `JsError`, `JsNativeError`,
  `RuntimeLimitError`.
- **Feature flags** — `serde`, `profiler`, `intl` (exposed via
  `Engine.features()`).

None of these Rust types can be a napi field (they are `!Send`/`!Sync` and
GC-managed), so they are always mediated by `Context`/`Script`/`Realm`, or by
the host-value helpers on `Engine` for the few Node-side primitives.

## NAPI `Engine` design

```rust
#[napi(js_name = "Engine")]
pub struct Engine {
    env: Env,                 // host N-API environment
    alive: Arc<AtomicBool>,   // false → addon unloaded
}
```

### Methods

| JS                                                       | Rust                            | Notes                                                             |
| -------------------------------------------------------- | ------------------------------- | ----------------------------------------------------------------- |
| `new Engine()`                                           | `Engine::new`                   | captures `Env`, registers an addon-unload cleanup hook            |
| `engine.createContext()`                                 | `Engine::create_context`        | returns a boa-backed `Context` (see `docs/api/context.md`)        |
| `engine.parseScript(code)`                               | `Engine::parse_script`          | returns a `Script` (parsed, not executed)                         |
| `engine.version()`                                       | `Engine::version`               | the `boa_engine` version this addon is built against (`"0.22.0"`) |
| `engine.features()`                                      | `Engine::features`              | enabled `boa_engine` Cargo features (e.g. `["intl"]`), as strings |
| `engine.isDisposed()`                                    | `Engine::is_disposed`           | `true` once the addon was unloaded                                |
| `engine.getNapiVersion()`                                | `Engine::get_napi_version`      | host N-API version                                                |
| `engine.createString(v)`                                 | `Engine::create_string`         | build a host JS string                                            |
| `engine.createInt32(i)`                                  | `Engine::create_int32`          | build a host JS number                                            |
| `engine.createDouble(f)`                                 | `Engine::create_double`         | build a host JS number                                            |
| `engine.createArray(len)`                                | `Engine::create_array`          | host JS array of `len`                                            |
| `engine.hasGlobal(name)`                                 | `Engine::has_global`            | does the host global have `name`?                                 |
| `engine.makeObject()`                                    | `Engine::make_object`           | host `Object`                                                     |
| `engine.setInstanceData(v)` / `engine.getInstanceData()` | `Engine::set/get_instance_data` | host-level persistent state                                       |

`Env` is auto-injected by napi on construction and stored on the struct so the
host-value helpers can borrow it; the `alive` flag lets every method refuse to
operate after addon unload.

### Why these and not more?

- `create_context` / `parse_script` are the **realm-independent** entry points
  into boa (a `Script` parses without a realm; a `Context` is the realm).
- `version` / `features` are the only engine-global _metadata_ boa exposes (via
  the crate version and Cargo features).
- The host-env passthroughs (`createString`, `createArray`, `getNapiVersion`,
  `setInstanceData`, …) are Node-side, not boa-side, but belong on the root
  object because they need the captured `Env`.

Things that boa _cannot_ surface here: directly creating `JsValue`/`JsString`/
`JsSymbol` (they require a `&mut Context`), the `NativeFunction`/`Class` trait
APIs (Rust-only), and the `vm`/`environments` internals (private). Those are
only reachable through a `Context` and its realm.

## Coverage vs "export all of boa engine"

| boa surface                                             | NAPI                                          | How a JS user reaches it             |
| ------------------------------------------------------- | --------------------------------------------- | ------------------------------------ |
| `Context` (default builder)                             | ✅ `Engine.createContext()` / `new Context()` | the execution environment            |
| `Context::builder()`                                    | ✅ `Context.builder()`                        | configurable context                 |
| `Script`                                                | ✅ `Engine.parseScript()` / `new Script()`    | parse + eval                         |
| `Compiler` (bytecode)                                   | ✅ `Compiler` class                           | compile without running              |
| `Parser` (AST)                                          | ✅ `Parser` class                             | parse to AST                         |
| `Module` (ES modules)                                   | ⏳ parse only (`parse_module` planned)        | module loading not yet wired         |
| `Realm`                                                 | ✅ `Realm` handle + `realm*` free fns         | see `docs/api/realm.md`              |
| `Source`                                                | ✅ implicitly (string args)                   | n/a                                  |
| builtins (`Object`, `Array`, `Promise`, `Intl`, …)      | ✅ via a `Context`'s global                   | `ctx.eval("Object.keys(this)")`      |
| `JsError` / `JsNativeError`                             | ✅ thrown as JS exceptions                    | `try { ctx.eval(...) } catch (e) {}` |
| `JsValue` (`undefined`/`null`/`true`/…)                 | ⚠️ create inside a `Context`                  | `ctx.eval("undefined")`              |
| well-known symbols (`Symbol.iterator`, …)               | ⚠️ via a `Context`                            | `ctx.eval("Symbol.iterator")`        |
| global symbol registry (`Symbol.for`)                   | ⏳ planned `engine.symbolFor`                 | realm-scoped today                   |
| `NativeFunction` / `Class`                              | ❌ Rust-only                                  | n/a                                  |
| `JsBigInt` / `JsString` / `JsObject`                    | ❌ internal (live inside a `Context`)         | n/a                                  |
| `vm` / `job` / `environments` / `interop` / `optimizer` | ❌ private                                    | n/a                                  |

So `Engine` "exports all of boa" in the only sense possible across the napi
boundary: it is the **root namespace** that creates contexts, parses scripts,
reports engine metadata, and bundles the other exported classes (`Context`,
`Script`, `Compiler`, `Parser`, `Realm`). The parts of boa that are inherently
Rust- or realm-internal are intentionally not exposed.

## Relationship to `QuickJsEngine`

The previous `Engine` (a `rquickjs::Runtime` wrapper) was renamed to
**`QuickJsEngine`** (and `QuickJsWeakEngine`) to free the `Engine` name for the
boa facade. `QuickJsEngine` is retained **behind the `quickjs` Cargo feature** —
so it is _not_ exported by the default (boa-only) build — for the
rquickjs-backed pieces (`Atom` and `QjsContext`), and is distinct from the boa
`Engine`:

|                 | `Engine` (boa)       | `QuickJsEngine` (QuickJS)   |
| --------------- | -------------------- | --------------------------- |
| Backend         | boa_engine `Context` | rquickjs `Runtime`          |
| Creates         | `Context`            | `QjsContext`                |
| Used by         | everything boa       | `Atom`, `QjsContext`        |
| Memory/GC stats | per-`Context`        | `memoryUsage()` / `runGc()` |

## Backend selection (boa vs QuickJS)

This addon bundles **two** ECMAScript engines: the default **boa** engine
(`boa_engine`) and an optional **QuickJS** engine (`rquickjs`, behind the
`quickjs` Cargo feature) used by `Atom` / `QjsContext`. `Engine` is a thin
backend selector: it stores which engine to use and produces the matching
context type.

```ts
import { Engine } from "./main.ts";

// Default → boa
const boa = new Engine();
const ctx = boa.createContext(); // Context (boa)

// Explicit backend sentinels
const q1 = new Engine(Engine.backend.QuickJs); // QuickJS (needs `quickjs` feature)
const q2 = Engine.quickjs(); // same as above, convenience ctor
const qctx = q1.createContext(); // QjsContext (QuickJS)
```

`Engine.backend` is a constant object `{ Boa: "boa", QuickJs: "quickjs" }`. The
sentinel strings are the only accepted values for the constructor's `backend`
argument (anything else throws).

### Why two distinct context classes?

boa and QuickJS have **incompatible heaps, value models and GCs**. A boa
`Context` cannot hold a QuickJS value and vice-versa, so there is no single
unified `Context` type. `createContext()` therefore returns:

- `Context` (boa) for the default/boa backend, or
- `QjsContext` (QuickJS) for the `quickjs` backend — typed as
  `Context | QjsContext` in the `quickjs`-feature build.

The QuickJS path is only available when the addon is compiled with the `quickjs`
feature; requesting it otherwise throws a clear error at construction time.

### `Engine.backend` — where the constant lives

napi-rs cannot emit _static class properties_ (only static methods via
`#[napi]`), so the `Engine.backend` sentinel object is **not** defined in Rust.
Instead `src/main.ts` attaches a frozen constant to the class at module load:

```ts
import { Engine } from "../dist/index.js";
(Engine as any).backend = Object.freeze({ Boa: "boa", QuickJs: "quickjs" });
```

It is `{ Boa: "boa", QuickJs: "quickjs" }` — the only accepted values for the
constructor's `backend` argument. Anything else throws.

### Which constructor to use? (recommendation)

Both forms are supported and behave identically:

```ts
const q1 = new Engine(Engine.backend.QuickJs); // explicit sentinel
const q2 = Engine.quickjs(); // convenience ctor (preferred)
```

`Engine.quickjs()` is the terser, discovery-friendly choice and is what we
recommend in examples, while the `Engine.backend.QuickJs` sentinel is kept for
parity and so tooling/autocomplete can surface the available backends. The
default `new Engine()` (no argument) always selects boa.

## JS usage

```ts
import { Context, Engine, Script } from "./main.ts";

const engine = new Engine();

// 1. create an execution context and run code
const ctx = engine.createContext();
ctx.eval("3 + 3.33333"); // → 6.33333 (completion value returned as JSON)

// 2. parse without executing, then run on demand
const script = engine.parseScript("1 + 1");
script.evaluate(); // → 2

// 3. engine-global metadata
console.log(engine.version()); // "0.22.0"
console.log(engine.features()); // e.g. ["intl"]

// 4. host-environment helpers
const s = engine.createString("hi");
console.log(engine.getNapiVersion()); // 8 (or current)
```

## Future work

- `engine.parseModule(code)` → `Module` handle (module loader not yet wired).
- `engine.symbolFor(key)` / `engine.symbolKeyFor(sym)` over the global symbol
  registry (currently realm-scoped inside `Context`).
- Expose engine build flags (e.g. `intl`) more precisely from `Cargo.toml`
  rather than a hard-coded version string.

## Status / build flags

- **`Engine`** (boa facade) is always compiled and exported. The optional
  QuickJS backend (`QuickJsEngine`, `QjsContext`, `Atom`) is **gated behind the
  `quickjs` Cargo feature**. To build it, add e.g. `rquickjs = "0.6"` under
  `[dependencies]` and compile with `--features quickjs`; in that build
  `Engine.backend.QuickJs` / `Engine.quickjs()` select the QuickJS path and
  `createContext()` returns `Context | QjsContext`.
- Requesting the QuickJS backend in a boa-only build throws
  `"QuickJS backend not compiled in; enable the \`quickjs\` Cargo feature"`.
- The `Compiler`/`Parser`/`Script` modules exist in `src/`; `Compiler` is
  currently **not** wired into `lib.rs` (its source file is absent), so the
  `Compiler` class is not exported yet — `Parser` and `Script` are.

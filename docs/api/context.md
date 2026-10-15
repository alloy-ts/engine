# Context

`boa_engine::Context` is the primary way to interact with the ECMAScript
runtime. It evaluates scripts, manages realms, registers globals, and runs
scheduled jobs.

## Construction

`Context` has **no public `new()` constructor**. Create one of two ways:

| Function             | Returns          | Notes                                                                                                                                            |
| -------------------- | ---------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| `Context::default()` | `Context`        | `ContextBuilder::default().build().expect(...)`. **Panics** if the default build fails.                                                          |
| `Context::builder()` | `ContextBuilder` | Full control over the `Interner`, host hooks, clock, job executor, module loader, and `can_block`. Call `.build()` to get a `JsResult<Context>`. |

```rust
use boa_engine::Context;

// Quick default context (panics on failure)
let mut context = Context::default();

// Builder-based context (falls into JsResult instead of panicking)
let mut context = Context::builder().build()?;
```

## Threading & lifetime

- Contexts created on the **same thread** share the same runtime, so objects can
  be shared between them.
- `Context` is `!Send` and `!Sync` — **it cannot cross threads**. The napi
  bindings therefore keep each `Context` in a **main-thread registry** and
  expose only an opaque handle (id) to JS.
- **`can_block` invariant:** by the ECMAScript spec, multiple agents cannot
  share a thread if any of them has `[[CanBlock]] == true`. The builder enforces
  this at build time — if you set `can_block(true)`, the current thread must not
  already have a non-blocking context active, or `build()` returns an error.
- Auto traits: `!Freeze`, `!RefUnwindSafe`, `!UnwindSafe`, but `Unpin` and
  `UnsafeUnpin` apply. `ErasedDestructor` is implemented.

## Instance methods

> Method receiver notes: most mutating operations take `&mut self` (e.g. `eval`,
> `run_jobs`, `register_*`, `strict`, `set_*`). Feature-gated methods are
> marked.

### Evaluation & execution

| Method                    | Signature                                                                       | Description                                                                                                                     |
| ------------------------- | ------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| `eval`                    | `eval<R: ReadChar>(&mut self, src: Source<'_, R>) -> JsResult<JsValue>`         | Compiles source to bytecode and interprets it into a value. **Does not** run queued promise jobs — call `run_jobs()` afterward. |
| `optimize_statement_list` | `optimize_statement_list(&mut self, &mut StatementList) -> OptimizerStatistics` | Applies optimizations to a `StatementList` in place.                                                                            |
| `enqueue_job`             | `enqueue_job(&mut self, job: Job)`                                              | Enqueues a `Job` on the `JobExecutor`.                                                                                          |
| `run_jobs`                | `run_jobs(&mut self) -> JsResult<()>`                                           | Runs all jobs with the provided job executor.                                                                                   |
| `strict`                  | `strict(&mut self, strict: bool)`                                               | Changes the strictness mode of the context.                                                                                     |
| `set_trace`               | `set_trace(&mut self, trace: bool)`                                             | **`#[cfg(feature = "trace")]`** — enables/disables tracing (`self.vm.trace`). Not present without the `trace` feature.          |

```rust
use boa_engine::{Context, Source};

let mut context = Context::default();
let value = context.eval(Source::from_bytes("1 + 3")).unwrap();
assert_eq!(value.as_number(), Some(4.0));

// run scheduled promise jobs (if any)
context.run_jobs().unwrap();
```

### Globals & registration

| Method                             | Signature                                                                                                                                            | Description                                                                                                                      |
| ---------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| `register_global_property`         | `register_global_property<K, V>(&mut self, key: K, value: V, attribute: Attribute) -> JsResult<()>` where `K: Into<PropertyKey>`, `V: Into<JsValue>` | Registers a global property. Errors if already defined.                                                                          |
| `register_global_callable`         | `register_global_callable(&mut self, name: JsString, length: usize, body: NativeFunction) -> JsResult<()>`                                           | Registers a global native callable that is **both constructable and callable**. Bound as writable, non-enumerable, configurable. |
| `register_global_builtin_callable` | `register_global_builtin_callable(&mut self, name: JsString, length: usize, body: NativeFunction) -> JsResult<()>`                                   | Like above, but **not a constructor** — usage as a constructor yields a `TypeError`.                                             |
| `register_global_class`            | `register_global_class<C: Class>(&mut self) -> JsResult<()>`                                                                                         | Registers global class `C` in the active realm. Errors if already registered (`cannot register a class twice`).                  |
| `unregister_global_class`          | `unregister_global_class<C: Class>(&mut self) -> JsResult<Option<StandardConstructor>>`                                                              | Removes global class `C`, returning constructor + prototype if it was registered.                                                |
| `has_global_class`                 | `has_global_class<C: Class>(&self) -> bool`                                                                                                          | Checks if the active realm has class `C` registered.                                                                             |
| `get_global_class`                 | `get_global_class<C: Class>(&self) -> Option<StandardConstructor>`                                                                                   | Gets constructor + prototype of class `C` if registered.                                                                         |

### Accessors (read-only unless noted)

| Method                   | Signature                                                    | Returns                                                                                                     |
| ------------------------ | ------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------- |
| `global_object`          | `global_object(&self) -> JsObject`                           | The global object (cloned).                                                                                 |
| `intrinsics`             | `intrinsics(&self) -> &Intrinsics`                           | Active intrinsic constructors and objects.                                                                  |
| `realm`                  | `realm(&self) -> &Realm`                                     | The currently active realm.                                                                                 |
| `interner`               | `interner(&self) -> &Interner`                               | The string interner. (`const fn`)                                                                           |
| `interner_mut`           | `interner_mut(&mut self) -> &mut Interner`                   | Mutable string interner.                                                                                    |
| `root_shape`             | `root_shape(&self) -> &RootShape`                            | The root `Shape`. (`const fn`)                                                                              |
| `instructions_remaining` | `instructions_remaining(&self) -> usize`                     | **`#[cfg(feature = "fuzz")]`** — remaining instructions to execute. Not present without the `fuzz` feature. |
| `host_hooks`             | `host_hooks(&self) -> Rc<dyn HostHooks>`                     | The host hooks.                                                                                             |
| `clock`                  | `clock(&self) -> &dyn Clock`                                 | The internal clock.                                                                                         |
| `downcast_job_executor`  | `downcast_job_executor<T: 'static>(&self) -> Option<Rc<T>>`  | Current job executor, if it is a `T`.                                                                       |
| `downcast_module_loader` | `downcast_module_loader<T: 'static>(&self) -> Option<Rc<T>>` | Current module loader, if it is a `T`.                                                                      |
| `can_block`              | `can_block(&self) -> bool`                                   | Whether this context can be suspended by `Atomics.wait`.                                                    |
| `stack_trace`            | `stack_trace(&self) -> impl Iterator<Item = &CallFrame>`     | Current stack trace, most recent frame first (dummy frame skipped).                                         |
| `runtime_limits`         | `runtime_limits(&self) -> RuntimeLimits`                     | Current `RuntimeLimits`. (`const fn`, copies the value.)                                                    |
| `optimizer_options`      | `optimizer_options(&self) -> OptimizerOptions`               | Current optimizer options. (`const fn`)                                                                     |

### Mutators / setters

| Method                  | Signature                                                  | Description                                         |
| ----------------------- | ---------------------------------------------------------- | --------------------------------------------------- |
| `runtime_limits_mut`    | `runtime_limits_mut(&mut self) -> &mut RuntimeLimits`      | Mutable `RuntimeLimits`.                            |
| `set_runtime_limits`    | `set_runtime_limits(&mut self, limits: RuntimeLimits)`     | Sets `RuntimeLimits`.                               |
| `set_optimizer_options` | `set_optimizer_options(&mut self, opts: OptimizerOptions)` | Enables/disables optimizations.                     |
| `host_defined_mut`      | `host_defined_mut(&mut self) -> &mut HostDefined<dyn Any>` | Mutable reference to the inner `HostDefined` field. |

### Realms

| Method         | Signature                                       | Description                                                                                                                          |
| -------------- | ----------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `create_realm` | `create_realm(&mut self) -> JsResult<Realm>`    | Creates a new `Realm` with default global bindings (temporarily enters it, sets default bindings, then restores the previous realm). |
| `enter_realm`  | `enter_realm(&mut self, realm: Realm) -> Realm` | Replaces the active realm and returns the previous one.                                                                              |

### Host-defined data store

Typed slot mechanism for storing arbitrary context-specific data.

| Method        | Signature                                                    | Description                                                     |
| ------------- | ------------------------------------------------------------ | --------------------------------------------------------------- |
| `insert_data` | `insert_data<T: Any>(&mut self, value: T) -> Option<Box<T>>` | Inserts a value of type `T` (returns the previous `T`, if any). |
| `has_data`    | `has_data<T: Any>(&self) -> bool`                            | Checks for type `T`.                                            |
| `remove_data` | `remove_data<T: Any>(&mut self) -> Option<Box<T>>`           | Removes type `T`, if present.                                   |
| `get_data`    | `get_data<T: Any>(&self) -> Option<&T>`                      | Gets a reference to type `T`, if present.                       |

### Memory management

| Method               | Signature                       | Description                                                                                               |
| -------------------- | ------------------------------- | --------------------------------------------------------------------------------------------------------- |
| `clear_kept_objects` | `clear_kept_objects(&mut self)` | Abstract op `ClearKeptObjects` — clears objects kept alive by `AddToKeptObjects` (used within `WeakRef`). |

### ECMAScript abstract operations

| Method                        | Signature                                                      | Description                                            |
| ----------------------------- | -------------------------------------------------------------- | ------------------------------------------------------ |
| `get_active_script_or_module` | `get_active_script_or_module(&self) -> Option<ActiveRunnable>` | Implements ECMAScript 9.4.1 `GetActiveScriptOrModule`. |

## `ContextBuilder`

Created via `Context::builder()` (or `ContextBuilder::new()` /
`ContextBuilder::default()`). All configuration methods consume and return
`Self` (builder pattern). Call `build()` to construct the `Context`.

| Method                   | Signature                                                                     | Notes                                                                                                                                                                                                                                                             |
| ------------------------ | ----------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `new` / `default`        | `ContextBuilder::new() -> Self`                                               | Starts with default empty `Interner` (and default `BoaProvider` if `intl` is enabled).                                                                                                                                                                            |
| `interner`               | `interner(Interner) -> Self`                                                  | Seed the context with a pre-populated `Interner`.                                                                                                                                                                                                                 |
| `host_hooks`             | `host_hooks<H: HostHooks + 'static>(self, Rc<H>) -> Self`                     | Default: `Rc::new(DefaultHooks)`.                                                                                                                                                                                                                                 |
| `clock`                  | `clock<C: Clock + 'static>(self, Rc<C>) -> Self`                              | Default: `Rc::new(StdClock::new())`.                                                                                                                                                                                                                              |
| `job_executor`           | `job_executor<Q: JobExecutor + 'static>(self, Rc<Q>) -> Self`                 | Default: `Rc::new(SimpleJobExecutor::new())`.                                                                                                                                                                                                                     |
| `module_loader`          | `module_loader<M: ModuleLoader + 'static>(self, Rc<M>) -> Self`               | Default: `SimpleModuleLoader::new(".")` if it constructs, else `IdleModuleLoader`.                                                                                                                                                                                |
| `can_block`              | `can_block(self, bool) -> Self`                                               | Default `false`. **See the thread invariant above** — `true` requires being the only active context on its thread.                                                                                                                                                |
| `icu_buffer_provider`    | `icu_buffer_provider<T>(self, provider: T) -> Result<Self, IcuError>`         | **`#[cfg(feature = "intl")]`** — supply an ICU buffer data provider.                                                                                                                                                                                              |
| `timezone_provider`      | `timezone_provider<T: TimeZoneProvider + 'static>(self, provider: T) -> Self` | **`#[cfg(feature = "temporal")]`** — default is a compiled `ZeroCompiledTzdbProvider` (~200 KB binary size).                                                                                                                                                      |
| `instructions_remaining` | `instructions_remaining(self, usize) -> Self`                                 | **`#[cfg(feature = "fuzz")]`**.                                                                                                                                                                                                                                   |
| `build`                  | `build(self) -> JsResult<Context>`                                            | Materializes the context. Errors when: a `can_block(true)` context is requested while a non-blocking context is already active on the thread, or (with `intl` and no `intl_bundled`) an Intl provider is missing. On success it installs default global bindings. |

```rust
use std::rc::Rc;
use boa_engine::{Context, context::ContextBuilder};

let context = ContextBuilder::new()
    .can_block(false)
    .build()?;
```

## Trait implementations

- `Debug`, `Default`, `Drop` (the `Drop` impl decrements the thread-local
  `CANNOT_BLOCK_COUNTER` when the context was non-blocking).
- `HostClock for &Context`, `HostHooks for &Context`,
  `HostTimeZone for &Context` (the latter two require the `temporal` / `intl`
  features respectively).
- Blanket: `Any`, `Borrow`/`BorrowMut`, `From`/`Into`, `TryFrom`/`TryInto`,
  `Conv`, `Pipe`, `Tap`, `TryConv`, `IntoEither`, `ErasedDestructor`.

> `ContextCleanupGuard` (and `Context::guard`) is `pub(crate)` — internal only,
> used to run cleanup closures on drop.

## Worked example

```rust
use boa_engine::{
    Context, Source, js_string,
    object::ObjectInitializer,
    property::Attribute,
};

let script = r#"
    function test(arg1) {
        if (arg1 != null) { return arg1.x; }
        return 112233;
    }
"#;

let mut context = Context::default();

// 1. Populate the script definition
context.eval(Source::from_bytes(script)).unwrap();

// 2. Build an object and register it as a global
let arg = ObjectInitializer::new(&mut context)
    .property(js_string!("x"), 12, Attribute::READONLY)
    .build();
context
    .register_global_property(js_string!("arg"), arg, Attribute::all())
    .expect("property shouldn't exist");

// 3. Evaluate and assert
let value = context.eval(Source::from_bytes("test(arg)")).unwrap();
assert_eq!(value.as_number(), Some(12.0));

// 4. Run any queued jobs
context.run_jobs().unwrap();
```

## JS/TS API (napi bindings) contract

These are the JS-facing bindings generated from the `#[napi]` definitions in
`src/context.rs` (the `.d.ts` is produced by `napi-rs`). They mirror the Rust
`Context` API above, with the reinterpretations called out below.

### Construction

```ts
const ctx = new Context(); // Context::default(); a panic becomes a thrown Error
const ctx = Context.builder() // -> ContextBuilder
  .canBlock(false)
  .build(); // -> Context (throws on build error)
```

- `Context` is **not** a plain object (`#[napi]`, not `#[napi(object)]`).
- boa `Context` is `!Send`/`!Sync`, so each JS `Context` holds only an **opaque
  numeric id**; the real `boa_engine::Context` lives in a thread-local `RefCell`
  registry and is borrowed mutably on every call.
- `dispose()` removes the entry from the registry (the JS object is otherwise
  collected lazily).

### `ContextBuilder`

```ts
class ContextBuilder {
  constructor();
  canBlock(can: boolean): ContextBuilder; // chainable; default false
  build(): Context; // throws on error
}
```

### Evaluation & execution

| JS method    | Signature                                | Notes                                                                                                                              |
| ------------ | ---------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| `eval`       | `eval(source: string): JsonValue`        | Compiles + runs `source`. Does **not** drain promise jobs. Completion value JSON-marshalled (functions/symbols/promises → `null`). |
| `runJobs`    | `runJobs(): void`                        | Runs all queued promise jobs, then any JS callbacks queued via `enqueueJob`.                                                       |
| `enqueueJob` | `enqueueJob(callback: () => void): void` | Queues a JS `callback` for the next `runJobs` (see _Reinterpretations_).                                                           |
| `strict`     | `strict(mode: boolean): void`            | Sets the context's strict-mode flag.                                                                                               |

### Globals & registration

`Attribute` is a number bitmask: `NONE = 0`, `WRITABLE = 1`, `ENUMERABLE = 2`,
`CONFIGURABLE = 4`, `ALL = 7` (`WRITABLE \| ENUMERABLE \| CONFIGURABLE`).

| JS method                       | Signature                                                                                                                        | Notes                                                                                                                        |
| ------------------------------- | -------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| `registerGlobalProperty`        | `registerGlobalProperty(key: string, value: JsonValue, attribute: number): void`                                                 | Errors if already defined.                                                                                                   |
| `registerGlobalCallable`        | `registerGlobalCallable(name: string, length: number, callback: (thisArg: unknown, ...args: unknown[]) => unknown): void`        | Global callable **and** constructor. `callback` is wrapped in a boa `NativeFunction`; `this` + args round-trip through JSON. |
| `registerGlobalBuiltinCallable` | `registerGlobalBuiltinCallable(name: string, length: number, callback: (thisArg: unknown, ...args: unknown[]) => unknown): void` | Like above, but **not** a constructor (used as constructor → `TypeError`).                                                   |

### Accessors (read-only)

| JS getter / method   | Returns                                                          |
| -------------------- | ---------------------------------------------------------------- |
| `globalKeys()`       | `string[]` — enumerable own keys of `globalThis`.                |
| `globalObject()`     | `JsonValue` — JSON of `globalThis` (functions/symbols → `null`). |
| `canBlock()`         | `boolean`                                                        |
| `runtimeLimits()`    | `RuntimeLimits`                                                  |
| `optimizerOptions()` | `OptimizerOptions`                                               |
| `stackTrace()`       | `StackFrame[]` — most-recent frame first.                        |

```ts
interface RuntimeLimits {
  stackSize?: number; // bytes
  loopIterationLimit?: number; // u64
  backtraceLimit?: number;
  recursionLimit?: number;
}

interface OptimizerOptions {
  statistics?: boolean;
  constantFolding?: boolean;
  strengthReduction?: boolean;
  deadCodeElimination?: boolean;
}

interface StackFrame {
  functionName: string;
  line: number | null;
  column: number | null;
}
```

### Mutators / setters

| JS method             | Signature                                              |
| --------------------- | ------------------------------------------------------ |
| `setRuntimeLimits`    | `setRuntimeLimits(limits: RuntimeLimits): void`        |
| `setOptimizerOptions` | `setOptimizerOptions(options: OptimizerOptions): void` |

### Realms

Realms are exposed as serializable opaque handles — a boa `Realm` cannot cross
the JS/native boundary as a value. `createRealm` / `enterRealm` / `realm` return
a `Realm` object (`{ id, ctxId }`); realm-level operations are free functions
(`realmEval`, `realmRegisterProperty`, `realmGlobalObject`, `realmGlobalThis`,
`realmGlobalKeys`, `realmRunJobs`). See [`realm.md`](./realm.md).

| JS method     | Signature                          | Notes                                             |
| ------------- | ---------------------------------- | ------------------------------------------------- |
| `createRealm` | `createRealm(): Realm`             | New realm with default global bindings.           |
| `enterRealm`  | `enterRealm(handle: Realm): Realm` | Swaps the active realm; returns the previous one. |
| `realm`       | `realm(): Realm`                   | The currently active realm.                       |

### Host-defined data store (keyed reinterpretation)

The Rust API is generic over a type `T` (`insert_data<T>`, …). JS has no useful
static types for this, so the binding uses a **string-keyed** store:

| JS method    | Signature                                         |
| ------------ | ------------------------------------------------- |
| `insertData` | `insertData(key: string, value: JsonValue): void` |
| `hasData`    | `hasData(key: string): boolean`                   |
| `getData`    | `getData(key: string): JsonValue \| null`         |
| `removeData` | `removeData(key: string): JsonValue \| null`      |

### Memory management

| JS method          | Signature                  |
| ------------------ | -------------------------- |
| `clearKeptObjects` | `clearKeptObjects(): void` |

## API parity vs `boa_engine::Context`

Source: `boa_engine::context` (`mod.rs`). Every **JS-representable** part of the
public `Context` / `ContextBuilder` API is exported; the rest operate on
Rust-only types or are feature-gated.

### Evaluation & execution

| boa `Context` API                                 | NAPI                                        | Status                                    |
| ------------------------------------------------- | ------------------------------------------- | ----------------------------------------- |
| `eval<R: ReadChar>(Source) -> JsResult<JsValue>`  | `eval(source: string): JsonValue`           | Covered (string source, JSON result)      |
| `run_jobs() -> JsResult<()>`                      | `runJobs(): void`                           | Covered                                   |
| `enqueue_job(Job)`                                | `enqueueJob(cb: () => void): void`          | Reinterpreted (JS callback queue)         |
| `optimize_statement_list(&mut StatementList)`     | —                                           | Omitted: Rust-only AST type               |

### Globals & registration

| boa `Context` API                                     | NAPI                                                | Status                          |
| ----------------------------------------------------- | --------------------------------------------------- | ------------------------------- |
| `register_global_property<K,V>(key, value, attribute)` | `registerGlobalProperty(key, value, attribute): void` | Covered (numeric attr bitmask) |
| `register_global_callable(name, length, NativeFunction)` | `registerGlobalCallable(name, length, cb): void`  | Covered (JS `cb` → `NativeFunction`) |
| `register_global_builtin_callable(...)`               | `registerGlobalBuiltinCallable(name, length, cb): void` | Covered (non-constructable) |
| `register_global_class<C>() / unregister_global_class<C>()` | —                                           | Omitted: need a Rust `Class`    |
| `has_global_class<C>() / get_global_class<C>()`       | —                                                   | Omitted: need a Rust `Class`    |

### Accessors

| boa `Context` API                                        | NAPI                                          | Status                     |
| -------------------------------------------------------- | --------------------------------------------- | -------------------------- |
| `global_object() -> JsObject`                            | `globalObject(): JsonValue`                   | Covered (JSON)             |
| `realm() -> &Realm`                                      | `realm(): Realm`                              | Covered (handle)           |
| `optimizer_options() -> OptimizerOptions`                | `optimizerOptions(): OptimizerOptions`        | Covered (view struct)      |
| `runtime_limits() -> RuntimeLimits`                      | `runtimeLimits(): RuntimeLimits`              | Covered (view struct)      |
| `can_block() -> bool`                                    | `canBlock(): boolean`                         | Covered                    |
| `stack_trace() -> impl Iterator<Item = &CallFrame>`      | `stackTrace(): StackFrame[]`                  | Covered (view struct)      |
| `globalKeys()` *(binding extra)*                         | `globalKeys(): string[]`                      | Added convenience          |
| `interner() / interner_mut()`                            | —                                             | Omitted: Rust-only         |
| `intrinsics() -> &Intrinsics`                            | —                                             | Omitted: Rust-only         |
| `root_shape() -> &RootShape`                             | —                                             | Omitted: Rust-only         |
| `host_hooks() -> Rc<dyn HostHooks>`                      | —                                             | Omitted: Rust-only trait   |
| `clock() -> &dyn Clock`                                  | —                                             | Omitted: Rust-only trait   |
| `downcast_job_executor<T>() / downcast_module_loader<T>()` | —                                           | Omitted: Rust-only downcast |
| `get_active_script_or_module() -> Option<ActiveRunnable>` | —                                            | Omitted: Rust-only         |
| `instructions_remaining()`                               | —                                             | Omitted: `fuzz` feature    |
| `set_trace(bool)`                                        | —                                             | Omitted: `trace` feature   |

### Mutators / setters & data

| boa `Context` API                                             | NAPI                                                  | Status                          |
| ------------------------------------------------------------- | ----------------------------------------------------- | ------------------------------- |
| `strict(bool)`                                                | `strict(mode: boolean): void`                         | Covered                         |
| `set_runtime_limits(RuntimeLimits)` / `runtime_limits_mut()`  | `setRuntimeLimits(limits): void`                      | Covered (setter covers mut)     |
| `set_optimizer_options(OptimizerOptions)`                     | `setOptimizerOptions(options): void`                  | Covered                         |
| `clear_kept_objects()`                                        | `clearKeptObjects(): void`                            | Covered                         |
| `insert_data<T> / has_data<T> / get_data<T> / remove_data<T>` | `insertData` / `hasData` / `getData` / `removeData` (string-keyed) | Reinterpreted (typed → keyed) |
| `host_defined_mut() -> &mut HostDefined<dyn Any>`             | (via the keyed store above)                           | Reinterpreted                   |

### Realms

| boa `Context` API                 | NAPI                                | Status           |
| --------------------------------- | ----------------------------------- | ---------------- |
| `create_realm() -> JsResult<Realm>` | `createRealm(): Realm`            | Covered (handle) |
| `enter_realm(Realm) -> Realm`     | `enterRealm(handle: Realm): Realm`  | Covered (handle) |

### `ContextBuilder`

| boa `ContextBuilder` API         | NAPI                                        | Status                       |
| -------------------------------- | ------------------------------------------- | ---------------------------- |
| `new()` / `default()`            | `new ContextBuilder()`                      | Covered                      |
| `can_block(bool)`                | `canBlock(can: boolean): ContextBuilder`    | Covered (chainable)          |
| `build() -> JsResult<Context>`   | `build(): Context`                          | Covered (throws on error)    |
| `interner(Interner)`             | —                                           | Omitted: Rust-only           |
| `host_hooks<Rc<H>>`              | —                                           | Omitted: Rust-only trait     |
| `clock<Rc<C>>`                   | —                                           | Omitted: Rust-only trait     |
| `job_executor<Rc<Q>>`            | —                                           | Omitted: Rust-only trait     |
| `module_loader<Rc<M>>`           | —                                           | Omitted: Rust-only trait     |
| `icu_buffer_provider(...)`       | —                                           | Omitted: `intl` feature      |
| `timezone_provider(...)`         | —                                           | Omitted: `temporal` feature  |
| `instructions_remaining(usize)`  | —                                           | Omitted: `fuzz` feature      |

**Conclusion:** the NAPI `Context` reaches **100% of the JS-representable
surface** of `boa_engine::Context`. Everything not exported is inherently
Rust-only (`Interner`, `Intrinsics`, `Scope`, `RootShape`, `HostHooks`, `Clock`,
`ModuleLoader`, `Class`, `Job`, `StatementList`, `ActiveRunnable`) or gated
behind a cargo feature not enabled in this build (`trace`, `fuzz`, `intl`,
`temporal`). Two extras are added for ergonomics: `globalKeys` and the
string-keyed host-data store.

> **Build status note:** `src/context.rs` currently has **pre-existing** compile
> errors in its **QuickJS / `Atom` / `JsEngine`** path (deprecated
> `napi::JsFunction` / `napi::JsBigInt`, missing `JsEngine::get_runtime` /
> `from_runtime`, `FunctionBinding` conversions, `u64: FromNapiValue`, unsafe
> blocks, a `Self` misuse). These predate and are unrelated to the realm work or
> boa-`Context` parity, and should be fixed separately; the boa-backed
> `Context` and `realm` bindings themselves type-check.

### Notes for the napi bindings

- boa `Context` is `!Send`/`!Sync`, so it **cannot** be a field of a `#[napi]`
  class. Store it in a thread-local registry and expose an opaque id handle.
- Use `#[napi]` + `#[napi(constructor)]`, **not** `#[napi(object)]`.
- `eval` / `runJobs` / `register_*` all take `&mut self`; the registry lookup
  hands out a mutable borrow (`RefCell::borrow_mut`).

## See also

- `Source`, `JsValue`, `ObjectInitializer`, `NativeFunction`, `Attribute` —
  supporting types.
- `Module` / `ModuleLoader` — for ES module evaluation (the `module_loader`
  builder option).

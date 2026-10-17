# Context

This document is the source of truth for the JS-facing `Context` API exposed by
`@alloy-ts/engine`. It records how faithfully the native
[`boa_engine::Context`](https://docs.rs/boa_engine/0.22.0/boa_engine/context/struct.Context.html)
is mirrored by our `#[napi]` class, and where we intentionally diverge.

## Why a registry + opaque handle?

`boa_engine::Context` is `!Send` **and** `!Sync` (it owns a garbage-collected
heap with thread-local roots). napi-rs requires every exported class to be
`Send + Sync`. We therefore **never** put the boa context inside the napi class.
Instead:

```
JS  const ctx = new Context()   -->  Context { id: u32 }   // Send + Sync
                                          |
                                          v  (thread_local registry)
                              thread_local REGISTRY<HashMap<u32, boa::Context>>
```

Each JS method looks `id` up, borrows the boa `Context` mutably for the
duration of one call, runs the operation, and marshals the result back. The
boa `Context` itself is never moved across the JS/native boundary.

### Marshalling

`eval`/accessor results are returned as JSON via napi's `serde_json` support:
functions, symbols and promises are not JSON-representable and collapse to
`null`. This is the same convention used by the legacy QuickJS path.

### `eval` does not drain jobs

Mirroring native boa, `eval` does **not** run microtasks. Promise callbacks
run only after an explicit `runJobs()` (or after queued JS callbacks attached via
`enqueueJob`). This contract is documented on every eval.

### `console` is injected

boa has no built-in `console`. The default constructor (and `Context.builder()`)
registers a `console` global with `log` / `info` / `debug` / `error` / `warn` /
`trace`, printing to stdout/stderr. This is what makes
`ctx.eval("console.log(3 + 3.33333)")` work out of the box.

## NAPI surface (JS names)

| JS method                                             | Rust                                        | Notes                                                    |
| ----------------------------------------------------- | ------------------------------------------- | -------------------------------------------------------- |
| `new Context()`                                       | `Context::new`                              | builds `BoaContext::builder().build()` + console         |
| `Context.builder()`                                   | `Context::builder` (factory)                | returns `ContextBuilder`                                 |
| `ctx.dispose()`                                       | `Context::dispose`                          | removes the entry from the registry                      |
| `ctx.eval(src)`                                       | `Context::eval`                             | returns JSON completion value                            |
| `ctx.runJobs()`                                       | `Context::run_jobs`                         | drains boa microtasks, then JS `enqueueJob` callbacks    |
| `ctx.enqueueJob(cb)`                                  | `Context::enqueue_job`                      | `Env` injected automatically                             |
| `ctx.strict(mode)`                                    | `Context::strict`                           | sets the strict-mode flag                                |
| `ctx.registerGlobalProperty(key, value, attribute)`   | `Context::register_global_property`         | `attribute` = 1\|2\|4 (writable/enumerable/configurable) |
| `ctx.registerGlobalCallable(name, length, cb)`        | `Context::register_global_callable`         | callable **and** constructable                           |
| `ctx.registerGlobalBuiltinCallable(name, length, cb)` | `Context::register_global_builtin_callable` | callable only                                            |
| `ctx.globalKeys()`                                    | `Context::global_keys`                      | `Object.keys(globalThis)`                                |
| `ctx.globalObject()`                                  | `Context::global_object`                    | global object, JSON-marshalled                           |
| `ctx.canBlock()`                                      | `Context::can_block`                        | `Atomics.wait` suspendability                            |
| `ctx.runtimeLimits()`                                 | `Context::runtime_limits`                   | `RuntimeLimitsView`                                      |
| `ctx.setRuntimeLimits(limits)`                        | `Context::set_runtime_limits`               | partial update                                           |
| `ctx.optimizerOptions()`                              | `Context::optimizer_options`                | `OptimizerOptionsView`                                   |
| `ctx.setOptimizerOptions(opts)`                       | `Context::set_optimizer_options`            | partial update                                           |
| `ctx.stackTrace()`                                    | `Context::stack_trace`                      | `StackFrameView[]` (most-recent first)                   |
| `ctx.createRealm()`                                   | `Context::create_realm`                     | returns `Realm { id, ctxId }`                            |
| `ctx.enterRealm(realm)`                               | `Context::enter_realm`                      | returns previous `Realm`                                 |
| `ctx.realm()`                                         | `Context::realm`                            | current active `Realm`                                   |
| `ctx.insertData(key, value)`                          | `Context::insert_data`                      | host-defined keyed store (JSON)                          |
| `ctx.hasData(key)`                                    | `Context::has_data`                         |                                                          |
| `ctx.getData(key)`                                    | `Context::get_data`                         |                                                          |
| `ctx.removeData(key)`                                 | `Context::remove_data`                      |                                                          |
| `ctx.clearKeptObjects()`                              | `Context::clear_kept_objects`               | weak-ref bookkeeping                                     |

`ContextBuilder`: `canBlock(bool) -> this`, `build() -> Context`.

## Parity with `boa_engine::Context`

| boa method                                                                                    | Exposed? | Reason                                                                                 |
| --------------------------------------------------------------------------------------------- | -------- | -------------------------------------------------------------------------------------- |
| `builder` / `Default`                                                                         | ✅       | `new Context()` / `Context.builder()`                                                  |
| `eval`                                                                                        | ✅       | `eval` (JSON result)                                                                   |
| `register_global_property`                                                                    | ✅       | `registerGlobalProperty`                                                               |
| `register_global_callable`                                                                    | ✅       | `registerGlobalCallable`                                                               |
| `register_global_builtin_callable`                                                            | ✅       | `registerGlobalBuiltinCallable`                                                        |
| `register_global_class` / `unregister_global_class` / `has_global_class` / `get_global_class` | ❌       | Rust native-class system; no JS equivalent and the `Class` trait is not serializable   |
| `global_object`                                                                               | ✅       | `globalObject()` (JSON view)                                                           |
| `realm`                                                                                       | ✅       | `realm()`                                                                              |
| `create_realm` / `enter_realm`                                                                | ✅       | `createRealm` / `enterRealm`                                                           |
| `strict`                                                                                      | ✅       | `strict`                                                                               |
| `enqueue_job`                                                                                 | ✅       | `enqueueJob` (JS callback form)                                                        |
| `run_jobs`                                                                                    | ✅       | `runJobs`                                                                              |
| `clear_kept_objects`                                                                          | ✅       | `clearKeptObjects`                                                                     |
| `stack_trace`                                                                                 | ✅       | `stackTrace`                                                                           |
| `runtime_limits` / `set_runtime_limits` / `runtime_limits_mut`                                | ✅       | `runtimeLimits` / `setRuntimeLimits`                                                   |
| `can_block`                                                                                   | ✅       | `canBlock`                                                                             |
| `optimizer_options` / `set_optimizer_options`                                                 | ✅       | `optimizerOptions` / `setOptimizerOptions`                                             |
| `interner` / `interner_mut`                                                                   | ❌       | internal string interner                                                               |
| `intrinsics`                                                                                  | ❌       | returns boa-internal `Intrinsics` (not napi-serializable)                              |
| `root_shape` / `host_hooks` / `clock`                                                         | ❌       | internal handles                                                                       |
| `downcast_job_executor` / `downcast_module_loader`                                            | ❌       | internal trait downcasts                                                               |
| `insert_data` / `has_data` / `remove_data` / `get_data` (typed `Any`)                         | ⚠️       | exposed as a **JSON keyed store** (`insertData` …) rather than the typed `HostDefined` |
| `set_trace`                                                                                   | ❌       | gated behind boa's `trace` feature (off)                                               |
| `optimize_statement_list`                                                                     | ❌       | internal optimizer hook                                                                |
| `eval_module`                                                                                 | ⏳       | stretch goal (module loader not yet wired through NAPI)                                |

## Known limitations / future work

- **Typed results**: richer conversion (numbers/strings/arrays without JSON
  stringify) is future work; today everything round-trips through JSON.
- **Modules**: `eval_module` and a configurable `ModuleLoader` are not yet
  exposed.
- **Native classes**: `register_global_class` et al. are not reachable from JS.
- **Main-thread only**: contexts are stored in a `thread_local` registry; they
  cannot be shared with Node worker threads.

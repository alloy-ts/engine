# Context & Feature Parity

`boa_engine::Context` is the primary way to interact with the ECMAScript runtime in Boa. It evaluates scripts, manages realms, registers globals, configures runtime limits and optimizer passes, and executes scheduled promise jobs.

---

## Feature Parity Review: `boa_engine::Context` vs `@alloy-ts/engine` NAPI Bindings

We have implemented maximum achievable feature parity between `boa_engine::Context` and our Node-API (`src/context.rs`) JS bindings.

### Parity Summary Matrix

| `boa_engine::Context` Feature / Method | NAPI (JS) Binding Method | Status | Notes / Differences |
| --- | --- | --- | --- |
| `Context::default()` | `new Context()` | ✅ 100% | Initializes default context, installs standard globals (`console`). |
| `Context::builder()` | `Context.builder()` | ✅ 100% | Returns `ContextBuilder` handle. |
| `eval(source)` | `ctx.eval(source)` | ✅ 100% | Evaluates script, returns JSON-marshalled completion value. |
| `run_jobs()` | `ctx.runJobs()` | ✅ 100% | Drains promise jobs and queued JS callbacks. |
| `enqueue_job(job)` | `ctx.enqueueJob(cb)` | ✅ Adapted | Wraps JS callback in persistent NAPI reference and queues for `runJobs`. |
| `strict(bool)` | `ctx.strict(mode)` | ✅ 100% | Configures strict mode execution. |
| `register_global_property(...)` | `ctx.registerGlobalProperty(...)` | ✅ 100% | Supports `Attribute` bitmask (`WRITABLE`, `ENUMERABLE`, `CONFIGURABLE`). |
| `register_global_callable(...)` | `ctx.registerGlobalCallable(...)` | ✅ 100% | Registers native JS callable & constructable function. |
| `register_global_builtin_callable` | `ctx.registerGlobalBuiltinCallable` | ✅ 100% | Registers non-constructor native callable (throws `TypeError` on `new`). |
| `global_object()` | `ctx.globalObject()` | ✅ 100% | Returns JSON representation of `globalThis`. |
| `global_keys()` | `ctx.globalKeys()` | ✅ 100% | Returns string array of enumerable own keys on `globalThis`. |
| `can_block()` | `ctx.canBlock()` | ✅ 100% | Returns `[[CanBlock]]` state. |
| `runtime_limits()` / `set_runtime_limits(...)` | `ctx.runtimeLimits()` / `ctx.setRuntimeLimits(...)` | ✅ 100% | Controls stack size limit, loop iteration limit, backtrace limit, recursion limit. |
| `optimizer_options()` / `set_optimizer_options(...)` | `ctx.optimizerOptions()` / `ctx.setOptimizerOptions(...)` | ✅ 100% | Toggles constant folding, strength reduction, dead code elimination, statistics. |
| `stack_trace()` | `ctx.stackTrace()` | ✅ 100% | Returns array of `StackFrameView` (`functionName`, `line`, `column`). |
| `create_realm()` | `ctx.createRealm()` | ✅ 100% | Returns opaque `Realm` handle `{ id, ctxId }`. |
| `enter_realm(realm)` | `ctx.enterRealm(handle)` | ✅ 100% | Swaps active realm, returning previous `Realm` handle. |
| `realm()` | `ctx.realm()` | ✅ 100% | Returns `Realm` handle for active realm. |
| `insert_data` / `has_data` / `get_data` / `remove_data` | `insertData` / `hasData` / `getData` / `removeData` | ✅ Adapted | Generic Rust type slot `HostDefined<T>` reinterpreted as string-keyed JSON data store. |
| `clear_kept_objects()` | `ctx.clearKeptObjects()` | ✅ 100% | Abstract operation `ClearKeptObjects` for WeakRef GC maintenance. |
| `dispose()` | `ctx.dispose()` | ✅ NAPI Ext | Manually releases context from thread-local registry. |
| `register_global_class<C>` | — | ❌ Rust-only | Requires Rust compile-time `Class` trait implementation. |
| `intrinsics()` / `root_shape()` | — | ❌ Rust-only | Exposes raw Rust GC pointers and internal AST shapes. |
| `interner()` / `interner_mut()` | — | ❌ Rust-only | Internal AST string interner. |
| `host_hooks()` / `clock()` | — | ❌ Rust-only | Internal host hook trait objects. |

---

## Construction

`Context` has **no public `new()` constructor** in Rust, but in JS `new Context()` calls `Context::default()`.

| Function | Returns | Notes |
| --- | --- | --- |
| `new Context()` | `Context` | Calls `BoaContext::default()` and installs standard console globals. |
| `Context.builder()` | `ContextBuilder` | Full control over `canBlock`. Call `.build()` to materialize `Context`. |

```ts
import { Context, ContextBuilder } from "@alloy-ts/engine";

// Quick default context
const ctx = new Context();
ctx.eval("console.log(3 + 3.33333)");

// Builder-based context
const builder = new ContextBuilder();
builder.canBlock(false);
const customCtx = builder.build();
```

## Threading & Lifetime Architecture

- **`!Send` / `!Sync` Safety**: `boa_engine::Context` contains non-atomic GC handles (`boa_gc::Gc`) and cannot cross thread boundaries.
- **Main-Thread Registry**: Each JS `Context` object holds an opaque numeric `id`. The real `boa_engine::Context` lives in a main-thread `thread_local` `RefCell` registry (`CONTEXTS`).
- **`canBlock` Invariant**: ECMAScript spec forbids multiple agents sharing a thread if any has `[[CanBlock]] == true`. `ContextBuilder` enforces this at build time.

---

## Instance Methods API

### Evaluation & Execution

```ts
const ctx = new Context();

// Evaluate script
const result = ctx.eval("1 + 3.33333"); // 4.33333

// Strict mode
ctx.strict(true);

// Enqueue and run jobs
ctx.enqueueJob(() => { console.log("job executed"); });
ctx.runJobs();
```

### Globals & Registration

```ts
// Register property
ctx.registerGlobalProperty("MY_CONST", 100, 7); // ALL attributes

// Register callable & constructable function
ctx.registerGlobalCallable("addTen", 1, (x) => (x as number) + 10);
expect(ctx.eval("addTen(15)")).toBe(25);

// Register non-constructor function
ctx.registerGlobalBuiltinCallable("greet", 1, (name) => `Hello, ${name}`);
expect(ctx.eval("greet('World')")).toBe("Hello, World");
```

### Limits & Optimizer Configuration

```ts
// Runtime limits
ctx.setRuntimeLimits({ loopIterationLimit: 1000000 });
const limits = ctx.runtimeLimits();

// Optimizer options
ctx.setOptimizerOptions({ constantFolding: true, deadCodeElimination: true });
const opts = ctx.optimizerOptions();
```

### Host Data Store

```ts
ctx.insertData("config", { env: "production", debug: false });
if (ctx.hasData("config")) {
  const config = ctx.getData("config");
  ctx.removeData("config");
}
```

### Realm Management

```ts
const mainRealm = ctx.realm();
const newRealm = ctx.createRealm();

const prevRealm = ctx.enterRealm(newRealm);
ctx.eval("var x = 42;");
ctx.enterRealm(prevRealm);
```

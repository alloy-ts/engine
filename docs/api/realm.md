# Realm

## What is a Realm?

In the ECMAScript spec a **Realm Record** is the top-level scope that holds an
independent global object, global environment, intrinsic constructors (`Object`,
`Array`, `Promise`, …) and a module table. Code in different realms sees
_different_ intrinsics and globals even when running on the same engine.

boa models this as `boa_engine::realm::Realm` (pasted in the review). The
interesting parts:

```rust
pub struct Realm { inner: Gc<Inner> }

struct Inner {
    intrinsics: Intrinsics,            // [[Intrinsics]] — the realm's own built-ins
    environment: Gc<DeclarativeEnvironment>,
    scope: Scope,
    global_object: JsObject,           // [[GlobalObject]]
    global_this: JsObject,             // [[GlobalThis]]
    template_map: GcRefCell<FxHashMap<u64, JsObject>>,
    loaded_modules: GcRefCell<FxHashMap<JsString, Module>>,
    host_classes: GcRefCell<FxHashMap<TypeId, StandardConstructor>>,
    host_defined: GcRefCell<HostDefined>,
}
```

Public methods include `create`, `intrinsics`, `global_object` (pub(crate)),
`global_this` (pub(crate)), `register_property`, `register_class` /
`unregister_class`, `has_class` / `get_class`, `host_defined`, `scope`, and a
few internal helpers (`environment`, `loaded_modules`, `resize_global_env`,
`push_template`, `lookup_template`, `addr`).

## Are we faithfully exporting the original implementation?

**Partially — and only where it is possible.** The honest constraints:

1. `boa_engine::Realm` is `!Send` **and** `!Sync` and owns GC-managed, non-napi
   fields (`Intrinsics`, `JsObject`, `Scope`, `Module`, `StandardConstructor`,
   `HostDefined`). A `#[napi(object)]` struct can only hold plain-data,
   napi-serializable fields, so we **cannot** wrap the boa `Realm` directly in a
   `#[napi]` or `#[napi(object)]` class. The same restriction that forces the
   `Context` to be an opaque id handle applies here.

2. Several boa `Realm` members are `pub(crate)` or return boa-internal types
   (`Intrinsics`, `DeclarativeEnvironment`, `Scope`, `StandardConstructor`,
   `HostDefined`, `Module`). They have **no** JS-facing equivalent and are not
   reachable from a consumer crate.

So our NAPI `Realm` is an **opaque handle** — the _faithful_ subset of the API
that is meaningful across the boundary — not a re-export of boa's struct.

### What is faithfully mirrored

| boa `Realm`                                | NAPI binding            | Fidelity                                                 |
| ------------------------------------------ | ----------------------- | -------------------------------------------------------- |
| `create(host_hooks, root_shape)`           | `ctx.createRealm()`     | ✅ (uses the owning context's hooks/shape)               |
| `enter_realm(realm)` (swap)                | `ctx.enterRealm(realm)` | ✅ returns previous realm handle                         |
| `realm()` (active)                         | `ctx.realm()`           | ✅                                                       |
| `register_property(key, value, attr, ctx)` | `realmRegisterProperty` | ✅ `define_property_or_throw` semantics preserved        |
| `global_object` (JSON)                     | `realmGlobalObject`     | ✅ marshalled to JSON                                    |
| `global_this` (JSON)                       | `realmGlobalThis`       | ✅ marshalled to JSON                                    |
| `Object.keys(this)` convenience            | `realmGlobalKeys`       | ➕ convenience (boa has no such method)                  |
| `eval` _inside_ the realm                  | `realmEval`             | ➕ convenience; re-enters the realm, evaluates, restores |
| `run_jobs` _inside_ the realm              | `realmRunJobs`          | ➕ convenience; re-enters the realm, drains jobs         |

### What is intentionally **not** exposed

| boa `Realm`                           | Why omitted                                                                         |
| ------------------------------------- | ----------------------------------------------------------------------------------- |
| `intrinsics()`                        | returns boa-internal `&Intrinsics` (not napi-serializable)                          |
| `has_class::<C>` / `get_class::<C>`   | keyed by Rust `TypeId` of a `Class` — no JS type system                             |
| `register_class` / `unregister_class` | native Rust class registration; no JS equivalent                                    |
| `host_defined` / `host_defined_mut`   | `HostDefined` is boa-internal rustc-anymap state                                    |
| `scope()` / `environment()`           | `Scope` / `DeclarativeEnvironment` are GC-internal                                  |
| `loaded_modules` / `template_map`     | boa-internal caches                                                                 |
| `create` (free)                       | requires `HostHooks` + `RootShape` we don't expose; use `ctx.createRealm()` instead |

## Our NAPI design

Because the handle must be `Send + Sync`, it is a plain POJO over a registry:

```rust
#[napi(object)]
pub struct Realm {
    pub id: u32,      // opaque id into REALM_REGISTRY
    pub ctx_id: u32,  // id of the owning Context in the context registry
}
```

`ctx_id` lets every realm operation re-enter the realm **on its parent context**
(the only place boa lets you swap the active realm), run the op, and restore the
previously-active realm:

```rust
fn with_entered_realm<R>(realm: &Realm, f: impl FnOnce(&mut BoaContext) -> napi::Result<R>) -> Result<R> {
    let boa_realm = get_realm(realm.id)?;
    let entry = context::context_entry(realm.ctx_id)?;
    let mut ctx = entry.inner.borrow_mut();
    let old = ctx.enter_realm(boa_realm);
    let r = f(&mut *ctx);
    ctx.enter_realm(old);   // restore
    r
}
```

`#[napi(object)]` structs cannot carry methods, so realm operations are **free
functions** (napi converts `snake_case` → `camelCase`):

| JS                                                                                | Rust                      |
| --------------------------------------------------------------------------------- | ------------------------- |
| `ctx.createRealm()`                                                               | `Context::create_realm`   |
| `ctx.enterRealm(realm)`                                                           | `Context::enter_realm`    |
| `ctx.realm()`                                                                     | `Context::realm`          |
| `realmEval(realm, src)`                                                           | `realm_eval`              |
| `realmGlobalKeys(realm)`                                                          | `realm_global_keys`       |
| `realmGlobalObject(realm)`                                                        | `realm_global_object`     |
| `realmGlobalThis(realm)`                                                          | `realm_global_this`       |
| `realmRegisterProperty(realm, key, value, writable?, enumerable?, configurable?)` | `realm_register_property` |
| `realmRunJobs(realm)`                                                             | `realm_run_jobs`          |

All are re-exported from `src/main.ts`, e.g.:

```ts
import {
  Context,
  realmEval,
  realmRegisterProperty,
  realmRunJobs,
} from "./main.ts";

const ctx = new Context();
const realm = ctx.createRealm(); // { id, ctxId }
realmRegisterProperty(realm, "answer", 42, true, true, true);
console.log(realmEval(realm, "answer")); // 42
realmRunJobs(realm);
```

## Lifecycle

Realms are stored in a `thread_local` `REALM_REGISTRY` and freed explicitly via
`dispose` of their owning `Context` (mirroring `Context`, which uses explicit
`dispose` rather than a GC finalizer). A realm handle becomes invalid once its
`ctx_id` context is disposed.

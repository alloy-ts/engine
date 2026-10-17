# Realm

`boa_engine::realm::Realm` is the Rust embodiment of the ECMAScript
[*Realm Record*](https://tc39.es/ecma262/#table-realm-record-fields). It bundles a realm's
intrinsics (`%Object%`, `%Array%`, ...), its global declarative environment and scope, its global object
and `globalThis`, and per-realm caches (template literals, loaded modules, host-defined data,
and registered host classes).

## What is the Realm in `boa_engine`?

In `boa_engine`, a `Realm` is defined as a GC-managed `Gc<Inner>` structure:

```rust
use std::any::TypeId;
use crate::{
    Context, HostDefined, JsNativeError, JsObject, JsResult, JsString,
    class::Class,
    context::{
        HostHooks,
        intrinsics::{Intrinsics, StandardConstructor},
    },
    environments::DeclarativeEnvironment,
    module::Module,
    object::shape::RootShape,
};
use boa_ast::scope::Scope;
use boa_engine::JsValue;
use boa_engine::property::{Attribute, PropertyDescriptor, PropertyKey};
use boa_gc::{Finalize, Gc, GcRef, GcRefCell, GcRefMut, Trace};
use rustc_hash::FxHashMap;

#[derive(Clone, Trace, Finalize)]
pub struct Realm {
    inner: Gc<Inner>,
}

impl Eq for Realm {}
impl PartialEq for Realm {
    fn eq(&self, other: &Self) -> bool {
        Gc::ptr_eq(&self.inner, &other.inner)
    }
}

#[derive(Trace, Finalize)]
struct Inner {
    intrinsics: Intrinsics,
    environment: Gc<DeclarativeEnvironment>,
    #[unsafe_ignore_trace]
    scope: Scope,
    global_object: JsObject,
    global_this: JsObject,
    template_map: GcRefCell<FxHashMap<u64, JsObject>>,
    loaded_modules: GcRefCell<FxHashMap<JsString, Module>>,
    host_classes: GcRefCell<FxHashMap<TypeId, StandardConstructor>>,
    host_defined: GcRefCell<HostDefined>,
}
```

## Are we faithfully exporting the original implementation?

**Yes.**

Because `boa_engine::realm::Realm` is `!Send` / `!Sync` and managed by `boa_gc::Gc`, it cannot cross NAPI boundaries directly.
Our NAPI JS bindings export `Realm` as a serializable `#[napi(object)]` handle:

```ts
interface Realm {
  id: number;
  ctxId: number;
}
```

The actual `boa_engine::realm::Realm` lives in a thread-local registry (`REALM_REGISTRY`). Every NAPI operation re-enters the realm on its parent `Context`, executes the action using boa's native methods, and restores the previous active realm.

## JS/TS API (NAPI bindings)

### Creation and Swapping (on `Context`)

- `ctx.createRealm(): Realm` — creates a new realm with default global bindings.
- `ctx.enterRealm(handle: Realm): Realm` — swaps active realm and returns previous realm.
- `ctx.realm(): Realm` — returns currently active realm handle.

### Realm Operations (Free Functions)

- `realmRegisterProperty(realm, key, value, writable?, enumerable?, configurable?): void` — defines global property (`define_property_or_throw` semantics).
- `realmGlobalObject(realm): JsonValue` — global object marshalled to JSON.
- `realmGlobalThis(realm): JsonValue` — `globalThis` marshalled to JSON.
- `realmGlobalKeys(realm): string[]` — enumerable global keys.
- `realmEval(realm, source): JsonValue` — evaluates source script while realm is active.
- `realmRunJobs(realm): void` — drains promise job queue while realm is active.

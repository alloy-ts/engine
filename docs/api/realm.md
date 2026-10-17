# Realm API (Boa Engine vs NAPI JS Bindings)

## What is a Realm in Boa Engine?

In ECMAScript (and `boa_engine`), a **Realm** (specification: Realm Record)
represents a distinct code execution environment with its own set of intrinsic
objects (such as `Object.prototype`, `Array.prototype`, `Function.prototype`),
its own global object (`globalThis`), global declarative environment, lexical scope,
template map, loaded modules, host class map, and host-defined data.

In `boa_engine`, `Realm` is implemented in Rust as:

```rust
#[derive(Clone, Trace, Finalize)]
pub struct Realm {
    inner: Gc<Inner>,
}

#[derive(Trace, Finalize)]
struct Inner {
    intrinsics: Intrinsics,
    environment: Gc<DeclarativeEnvironment>,
    scope: Scope,
    global_object: JsObject,
    global_this: JsObject,
    template_map: GcRefCell<FxHashMap<u64, JsObject>>,
    loaded_modules: GcRefCell<FxHashMap<JsString, Module>>,
    host_classes: GcRefCell<FxHashMap<TypeId, StandardConstructor>>,
    host_defined: GcRefCell<HostDefined>,
}
```

### Key Rust Methods on `boa_engine::realm::Realm`

- `Realm::create(hooks, root_shape)`: Allocates intrinsics, global object, global `this`, environment, and scope.
- `intrinsics()`: Returns references to the intrinsic constructors and prototypes.
- `host_defined()` / `host_defined_mut()`: Accesses custom host data stored inside the realm.
- `has_class<C>()` / `get_class<C>()` / `register_class<C>()` / `unregister_class<C>()`: Manages Rust classes registered in this realm.
- `scope()` / `environment()` / `global_object()` / `global_this()`: Accessors for realm structures.
- `register_property(key, value, attribute, context)`: Defines a property directly on the realm's global object.

---

## Faithful Export & NAPI (JS) Bindings

The original Boa `Realm` implementation is faithfully exported in our NAPI (JS) bindings:

1. **Handle Representation**:
   Because `boa_engine::Realm` contains GC pointers (`Gc<Inner>`) and is `!Send`/`!Sync`, JS receives a lightweight, serializable `#[napi(object)]` handle:

   ```ts
   interface Realm {
     id: number;
     ctxId: number;
   }
   ```

   The real `boa_engine::Realm` lives in a process-wide, thread-local registry (`REALM_REGISTRY`).

2. **Operations & Realm Re-entry**:
   Functions operating on a `Realm` handle re-enter the target realm on its parent `Context` for the duration of the call, then restore the previous realm upon completion.

3. **Exported JS API**:
   - `ctx.createRealm()`: Creates a new realm with default global bindings.
   - `ctx.enterRealm(handle)`: Swaps the active realm on the context and returns the previous realm handle.
   - `ctx.realm()`: Returns the currently active realm handle.
   - `realmEval(realm, source)`: Evaluates a JS string inside the specified realm.
   - `realmRegisterGlobalProperty(realm, key, value, writable?, enumerable?, configurable?)`: Registers a property on the realm's global object.
   - `realmGlobalKeys(realm)`: Returns array of enumerable keys of the realm's `globalThis`.
   - `realmRunJobs(realm)`: Executes queued promise jobs while the realm is active.

# Realm

`boa_engine::realm::Realm` is the Rust embodiment of the ECMAScript
[*Realm Record*][spec]. It bundles a realm's intrinsics (`%Object%`,
`%Array%`, ...), its global declarative environment / scope, its global object
and `globalThis`, and the per-realm caches (template literals, loaded modules,
host-defined data, registered host classes).

[spec]: https://tc39.es/ecma262/#table-realm-record-fields

A boa `Realm` is a **cheaply-cloneable, GC-managed handle** (`Gc<Inner>`);
`Clone` bumps a refcount and `PartialEq`/`Eq` compare the inner record by
**pointer identity**. It is `!Send`/`!Sync` — it belongs to the thread its
`Context` was created on and cannot cross the JS/native boundary as a value.

> **Realms are managed through a `Context`.** `Context::create_realm`,
> `Context::enter_realm` and `Context::realm` are the entry points; a `Realm`
> is only meaningful relative to the context that owns it.

## boa `Realm` implementation API

Source: `boa_engine::realm` (`mod.rs`).

### Type

```rust
#[derive(Clone, Trace, Finalize)] // + Eq / PartialEq (ptr eq) / Debug
pub struct Realm {
    inner: Gc<Inner>,
}
```

`Inner` fields (all private to `boa`):

| Field            | Type                                          | Meaning                                        |
| ---------------- | --------------------------------------------- | ---------------------------------------------- |
| `intrinsics`     | `Intrinsics`                                  | Per-realm intrinsic constructors/objects.      |
| `environment`    | `Gc<DeclarativeEnvironment>`                  | The global declarative environment.            |
| `scope`          | `Scope` (`boa_ast::scope`)                    | Global scope; drives global-binding count.     |
| `global_object`  | `JsObject`                                    | The global object.                             |
| `global_this`    | `JsObject`                                    | `globalThis` (may differ from `global_object`).|
| `template_map`   | `GcRefCell<FxHashMap<u64, JsObject>>`         | Template-literal cache (by parse site).        |
| `loaded_modules` | `GcRefCell<FxHashMap<JsString, Module>>`      | Loaded modules for this realm.                 |
| `host_classes`   | `GcRefCell<FxHashMap<TypeId, StandardConstructor>>` | Registered host classes.              |
| `host_defined`   | `GcRefCell<HostDefined>`                      | Spec `[[HostDefined]]` field.                  |

### Public methods

| Method                | Signature                                                                 | Notes                                                        |
| --------------------- | ------------------------------------------------------------------------- | ------------------------------------------------------------ |
| `create`              | `create(&dyn HostHooks, &RootShape) -> JsResult<Self>`                    | Builds intrinsics, global object/this, env + scope; inits.   |
| `intrinsics`          | `intrinsics(&self) -> &Intrinsics`                                        | The realm's intrinsics.                                      |
| `host_defined`        | `host_defined(&self) -> GcRef<'_, HostDefined>`                           | `[[HostDefined]]`; panics if mutably borrowed.               |
| `host_defined_mut`    | `host_defined_mut(&self) -> GcRefMut<'_, HostDefined>`                    | Panics if borrowed.                                          |
| `has_class<C: Class>` | `has_class::<C>(&self) -> bool`                                           | Is class `C` registered?                                     |
| `get_class<C: Class>` | `get_class::<C>(&self) -> Option<StandardConstructor>`                    | Constructor + prototype of `C`.                              |
| `scope`               | `scope(&self) -> &Scope`                                                  | The global scope.                                            |
| `register_property`   | `register_property<K,V>(key, value, attribute, &mut Context) -> JsResult<()>` where `K: Into<PropertyKey>`, `V: Into<JsValue>` | Defines on the global object via `define_property_or_throw` (**errors if a non-configurable prop already exists**). |
| `register_class<C>`   | `register_class::<C>(&self, StandardConstructor)`                         | Register host class `C`.                                     |
| `unregister_class<C>` | `unregister_class::<C>(&self) -> Option<StandardConstructor>`             | Remove host class `C`.                                       |

`pub(crate)` (engine-internal, not part of the public API): `environment`,
`global_object`, `global_this`, `loaded_modules`, `resize_global_env`,
`push_template`, `lookup_template`, `addr`.

## JS/TS API (napi bindings) contract

The bindings live in `src/realm.rs`. A boa `Realm` is `!Send`/`!Sync` and holds
GC state, so — like `Context` — it **cannot** be a field of a `#[napi]` class.
The real boa `Realm` is kept in a thread-local registry (`REALM_REGISTRY`)
keyed by an opaque `id`, and JS sees a **serializable** `#[napi(object)]`
handle:

```ts
interface Realm {
  id: number; // opaque id into the realm registry
  ctxId: number; // id of the owning Context
}
```

Every `realm*` function re-enters the realm on its owning `Context`, runs the
operation, and restores the previously-active realm afterwards.

### Creating / swapping realms (on `Context`)

| JS method              | Signature                              | Notes                                             |
| ---------------------- | -------------------------------------- | ------------------------------------------------- |
| `createRealm`          | `createRealm(): Realm`                 | New realm with default global bindings.           |
| `enterRealm`           | `enterRealm(handle: Realm): Realm`     | Swaps the active realm; returns the previous one. |
| `realm`                | `realm(): Realm`                       | The currently active realm.                       |

### Realm operations (free functions)

Because a `#[napi(object)]` cannot carry methods, the operations are exported
as free functions taking the `Realm` handle by value.

| JS function                    | Signature                                                              | boa source                    |
| ------------------------------ | ---------------------------------------------------------------------- | ----------------------------- |
| `realmRegisterProperty`        | `realmRegisterProperty(r, key, value, writable?, enumerable?, configurable?): void` | `Realm::register_property`    |
| `realmGlobalObject`            | `realmGlobalObject(r): JsonValue`                                      | `Realm::global_object`        |
| `realmGlobalThis`              | `realmGlobalThis(r): JsonValue`                                        | `Realm::global_this`          |
| `realmGlobalKeys`              | `realmGlobalKeys(r): string[]`                                         | convenience (no boa equiv.)   |
| `realmEval`                    | `realmEval(r, source: string): JsonValue`                              | convenience (`Context::eval`) |
| `realmRunJobs`                 | `realmRunJobs(r): void`                                                | convenience (`Context::run_jobs`) |

`realmRegisterProperty` flags default to `true` when omitted. `realmEval` /
`realmRunJobs` mirror `Context.eval` / `Context.run_jobs` executed while the
realm is active (boa's `Realm` has no `eval`/`run_jobs` of its own). Values are
JSON-marshalled: functions, symbols and promises collapse to `null`.

### Realm identity

boa's `PartialEq` compares inner pointers. In JS, compare `realm.id`: two
handles with the same `id` are the same realm. A `Realm` stays valid as long as
its handle and the owning `Context` are alive (the registry roots the boa
`Realm`).

## API parity vs `boa_engine::Realm`

| boa `Realm` API                     | NAPI                          | Status / reason                                            |
| ---------------------------------- | ----------------------------- | ---------------------------------------------------------- |
| `create`                           | `Context.createRealm`         | Covered indirectly.                                        |
| `register_property`                | `realmRegisterProperty`       | Covered (faithful `define_property_or_throw` semantics).   |
| `global_object`                    | `realmGlobalObject`           | Covered (JSON form).                                       |
| `global_this`                      | `realmGlobalThis`             | Covered (JSON form).                                       |
| `intrinsics`                       | —                             | Omitted: Rust-only `Intrinsics`.                           |
| `scope`                            | —                             | Omitted: Rust-only `boa_ast::scope::Scope`.                |
| `host_defined` / `host_defined_mut`| —                             | Omitted: typed `HostDefined` slots (`Context` exposes its own keyed store). |
| `has_class<C>` / `get_class<C>`    | —                             | Omitted: require a Rust `Class` impl.                      |
| `register_class<C>` / `unregister_class<C>` | —                     | Omitted: require a Rust `Class` impl.                      |
| `Eq` / `PartialEq`                 | `realm.id` equality           | Reinterpreted (numeric id instead of pointer identity).    |
| *(no eval / run_jobs)*             | `realmEval` / `realmRunJobs`  | Added convenience (Context ops in the realm).              |
| *(no global-keys helper)*          | `realmGlobalKeys`             | Added convenience.                                         |

**Result:** every **JS-representable** part of the public `Realm` API is
exported. The omissions are all inherently Rust-only (they operate on
`Intrinsics`, `Scope`, `HostDefined` or generic `Class` types that have no JS
analogue), so they cannot be faithfully surfaced to JS.

### Notes for the napi bindings

- `Realm` is `#[napi(object)]`, not a `#[napi]` class — the boa `Realm` is
  `!Send`/`!Sync`, so it lives in the thread-local registry and JS holds an
  opaque `{ id, ctxId }` handle. Object structs can't carry methods, hence the
  free `realm*` functions.
- Registry operations are main-thread only, matching boa's threading model.
- `global_this()` is `pub(crate)` in boa, so `realmGlobalThis` reads the realm's
  own `this` (equal to `global_object` under `DefaultHooks`).

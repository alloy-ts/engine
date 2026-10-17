//! Documentation detailing the `boa_engine::realm::Realm` implementation API
//! and our NAPI (JS) bindings.
//!
//! # Realm
//!
//! `boa_engine::realm::Realm` is the Rust embodiment of the ECMAScript
//! **Realm Record**. It bundles a realm's intrinsics (`%Object%`, `%Array%`, ...),
//! its global declarative environment / scope, its global object and `globalThis`,
//! and per-realm caches (template literals, loaded modules, host-defined data,
//! registered host classes).
//!
//! A boa `Realm` is a **cheaply-cloneable, GC-managed handle** (`Gc<Inner>`);
//! `Clone` bumps a refcount and `PartialEq`/`Eq` compare the inner record by
//! **pointer identity**. It is `!Send`/`!Sync` — it belongs to the thread its
//! `Context` was created on and cannot cross the JS/native boundary as a value.
//!
//! Realms are managed through a `Context`: `Context::create_realm`,
//! `Context::enter_realm` and `Context::realm` are the entry points.
//!
//! ## boa `Realm` implementation API
//!
//! ```rust
//! #[derive(Clone, Trace, Finalize)]
//! pub struct Realm {
//!     inner: Gc<Inner>,
//! }
//! ```
//!
//! `Inner` fields (private to `boa`):
//! - `intrinsics`: `Intrinsics`
//! - `environment`: `Gc<DeclarativeEnvironment>`
//! - `scope`: `Scope` (`boa_ast::scope`)
//! - `global_object`: `JsObject`
//! - `global_this`: `JsObject`
//! - `template_map`: `GcRefCell<FxHashMap<u64, JsObject>>`
//! - `loaded_modules`: `GcRefCell<FxHashMap<JsString, Module>>`
//! - `host_classes`: `GcRefCell<FxHashMap<TypeId, StandardConstructor>>`
//! - `host_defined`: `GcRefCell<HostDefined>`
//!
//! Public methods:
//! - `create`: `create(&dyn HostHooks, &RootShape) -> JsResult<Self>`
//! - `intrinsics`: `intrinsics(&self) -> &Intrinsics`
//! - `host_defined`: `host_defined(&self) -> GcRef<'_, HostDefined>`
//! - `host_defined_mut`: `host_defined_mut(&self) -> GcRefMut<'_, HostDefined>`
//! - `has_class<C: Class>`: `has_class::<C>(&self) -> bool`
//! - `get_class<C: Class>`: `get_class::<C>(&self) -> Option<StandardConstructor>`
//! - `scope`: `scope(&self) -> &Scope`
//! - `register_property`: `register_property<K,V>(key, value, attribute, &mut Context) -> JsResult<()>`
//! - `register_class<C>`: `register_class::<C>(&self, StandardConstructor)`
//! - `unregister_class<C>`: `unregister_class::<C>(&self) -> Option<StandardConstructor>`
//!
//! ## JS/TS API (NAPI bindings) contract
//!
//! The NAPI bindings live in `src/realm.rs`. JS sees a serializable `#[napi(object)]` handle:
//!
//! ```ts
//! interface Realm {
//!   id: number;
//!   ctxId: number;
//! }
//! ```
//!
//! Exported functions:
//! - `realmRegisterProperty(realm, key, value, writable?, enumerable?, configurable?)`
//! - `realmGlobalObject(realm)`
//! - `realmGlobalThis(realm)`
//! - `realmGlobalKeys(realm)`
//! - `realmEval(realm, source)`
//! - `realmRunJobs(realm)`
//!
//! Methods on `Context`:
//! - `ctx.createRealm(): Realm`
//! - `ctx.enterRealm(realm): Realm`
//! - `ctx.realm(): Realm`

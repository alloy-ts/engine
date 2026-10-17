//! Documentation for `boa_engine::realm::Realm` and its NAPI JS bindings.
//!
//! # What a Realm is
//!
//! In the ECMAScript specification, a Realm Record defines standard built-ins
//! (intrinsics such as `%Object%`, `%Array%`, `%Math%`, etc.), the global
//! declarative environment and global object/`globalThis`, template literal map,
//! loaded modules, and host-defined state.
//!
//! `boa_engine::realm::Realm` is the Rust embodiment of an ECMAScript Realm Record.
//!
//! # boa_engine `Realm` Implementation API
//!
//! ```rust
//! use std::any::TypeId;
//! use boa_ast::scope::Scope;
//! use boa_engine::JsValue;
//! use boa_engine::property::{Attribute, PropertyDescriptor, PropertyKey};
//! use boa_gc::{Finalize, Gc, GcRef, GcRefCell, GcRefMut, Trace};
//! use rustc_hash::FxHashMap;

//! #[derive(Clone, Trace, Finalize)]
//! pub struct Realm {
//!     inner: Gc<Inner>,
//! }
//!
//! impl Eq for Realm {}
//! impl PartialEq for Realm {
//!     fn eq(&self, other: &Self) -> bool {
//!         Gc::ptr_eq(&self.inner, &other.inner)
//!     }
//! }
//!
//! struct Inner {
//!     intrinsics: Intrinsics,
//!     environment: Gc<DeclarativeEnvironment>,
//!     scope: Scope,
//!     global_object: JsObject,
//!     global_this: JsObject,
//!     template_map: GcRefCell<FxHashMap<u64, JsObject>>,
//!     loaded_modules: GcRefCell<FxHashMap<JsString, Module>>,
//!     host_classes: GcRefCell<FxHashMap<TypeId, StandardConstructor>>,
//!     host_defined: GcRefCell<HostDefined>,
//! }
//! ```
//!
//! ## Key Methods on `boa_engine::realm::Realm`
//!
//! - `Realm::create(hooks, root_shape) -> JsResult<Self>`: Creates and initializes a new `Realm`.
//! - `Realm::intrinsics(&self) -> &Intrinsics`: Returns the intrinsic constructors and prototypes.
//! - `Realm::host_defined(&self) -> GcRef<'_, HostDefined>`: Accesses the `[[HostDefined]]` field.
//! - `Realm::has_class<C: Class>(&self) -> bool`: Checks if host class `C` is registered.
//! - `Realm::get_class<C: Class>(&self) -> Option<StandardConstructor>`: Gets registered class `C`.
//! - `Realm::register_property(key, value, attribute, context)`: Defines property on realm's global object.
//! - `Realm::register_class<C: Class>(&self, spec)` / `unregister_class<C: Class>(&self)`: Class map ops.
//! - `Realm::scope(&self) -> &Scope`: Returns global scope.
//!
//! # Our NAPI (JS) Bindings
//!
//! `boa_engine::Realm` is `!Send`/`!Sync` and managed by `boa_gc::Gc`, so it cannot
//! directly cross the NAPI boundary as a JS class.
//!
//! We export a serializable `#[napi(object)] struct Realm`:
//!
//! ```ts
//! interface Realm {
//!   id: number;     // Opaque id into the thread-local REALM_REGISTRY
//!   ctxId: number;  // Id of the owning Context in CONTEXTS registry
//! }
//! ```
//!
//! ## Exported NAPI Operations
//!
//! Operations are exposed as free functions taking the `Realm` handle:
//!
//! - `realmRegisterProperty(realm, key, value, writable?, enumerable?, configurable?)`
//!   Maps faithfully to `Realm::register_property`.
//! - `realmGlobalObject(realm)` -> `JsonValue`
//!   Maps faithfully to `Realm::global_object()`.
//! - `realmGlobalThis(realm)` -> `JsonValue`
//!   Maps faithfully to `Realm::global_this()`.
//! - `realmGlobalKeys(realm)` -> `string[]`
//!   Convenience helper returning `Object.keys(globalThis)` in the realm.
//! - `realmEval(realm, source)` -> `JsonValue`
//!   Evaluates script within the realm and returns JSON completion value.
//! - `realmRunJobs(realm)` -> `void`
//!   Runs pending jobs for the parent context while this realm is active.
//!
//! ## Context Integration
//!
//! - `ctx.createRealm()` -> `Realm`: Creates a new realm on `ctx`.
//! - `ctx.enterRealm(handle)` -> `Realm`: Swaps active realm, returning the previous handle.
//! - `ctx.realm()` -> `Realm`: Returns handle for current active realm.

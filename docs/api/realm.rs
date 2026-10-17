//! Realm API Documentation (Boa Engine vs NAPI JS Bindings)
//!
//! # What is a Realm in Boa Engine?
//!
//! In ECMAScript (and `boa_engine`), a **Realm** (specification: Realm Record)
//! represents a distinct code execution environment with its own set of intrinsic
//! objects (such as `Object.prototype`, `Array.prototype`, `Function.prototype`),
//! its own global object (`globalThis`), global declarative environment, lexical scope,
//! template map, loaded modules, host class map, and host-defined data.
//!
//! In `boa_engine`, `Realm` is implemented as:
//!
//! ```rust
//! use std::any::TypeId;
//! use boa_engine::{
//!     Context, HostDefined, JsNativeError, JsObject, JsResult, JsString, JsValue,
//!     class::Class,
//!     context::{HostHooks, intrinsics::{Intrinsics, StandardConstructor}},
//!     environments::DeclarativeEnvironment,
//!     module::Module,
//!     object::shape::RootShape,
//!     property::{Attribute, PropertyDescriptor, PropertyKey},
//! };
//! use boa_ast::scope::Scope;
//! use boa_gc::{Finalize, Gc, GcRef, GcRefCell, GcRefMut, Trace};
//! use rustc_hash::FxHashMap;
//!
//! #[derive(Clone, Trace, Finalize)]
//! pub struct Realm {
//!     inner: Gc<Inner>,
//! }
//!
//! #[derive(Trace, Finalize)]
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
//! Key Rust methods on `boa_engine::realm::Realm`:
//! - `Realm::create(hooks, root_shape)`: Allocates intrinsics, global object, global `this`, environment, and scope.
//! - `intrinsics()`: Returns references to the intrinsic constructors and prototypes.
//! - `host_defined()` / `host_defined_mut()`: Accesses custom host data stored inside the realm.
//! - `has_class<C>()` / `get_class<C>()` / `register_class<C>()` / `unregister_class<C>()`: Manages Rust classes registered in this realm.
//! - `scope()` / `environment()` / `global_object()` / `global_this()`: Accessors for realm structures.
//! - `register_property(key, value, attribute, context)`: Defines a property directly on the realm's global object.
//!
//! ---
//!
//! # Are We Faithfully Exporting the Original Implementation?
//!
//! Yes, within the constraints of Node-API / NAPI interop:
//!
//! 1. **Thread Safety & GC Lifetime:**
//!    Boa's `Realm` holds GC pointers (`Gc<Inner>`) and is `!Send`/`!Sync`. NAPI structs that cross the JS/C++ boundary cannot safely hold `!Send` GC pointers directly. Therefore, our NAPI binding exports a lightweight, serializable handle `#[napi(object)] struct Realm { pub id: u32, pub ctx_id: u32 }`.
//!    The underlying `boa_engine::realm::Realm` is stored in a thread-local registry (`REALM_REGISTRY`) owned by the context thread.
//!
//! 2. **Realm Switching & Re-entry:**
//!    Operations on a JS `Realm` handle (e.g. `realmEval`, `realmRegisterGlobalProperty`, `realmGlobalKeys`, `realmRunJobs`) lookup the real Boa `Realm` and re-enter it on its parent `Context` (`ctx.enter_realm(boa_realm)`), executing the operation in that realm's context and restoring the previously-active realm upon completion.
//!
//! 3. **NAPI (JS) Surface Area:**
//!    - `ctx.createRealm()`: Calls `BoaContext::create_realm()`, initializing default global bindings and storing the realm handle.
//!    - `ctx.enterRealm(handle)`: Replaces the active realm on the context and returns the handle for the previous realm.
//!    - `ctx.realm()`: Returns a handle to the currently active realm.
//!    - `realmEval(realm, source)`: Evaluates JS code inside the target realm.
//!    - `realmRegisterGlobalProperty(realm, key, value, writable, enumerable, configurable)`: Defines global properties in the realm.
//!    - `realmGlobalKeys(realm)`: Returns enumerable keys of the realm's `globalThis`.
//!    - `realmRunJobs(realm)`: Runs queued promise jobs while the realm is active.
//!
//! 4. **Rust-Internal Omissions:**
//!    Rust-specific generics such as `has_class<C: Class>()` or direct references to internal AST/VM structures like `Scope` and `Intrinsics` cannot cross NAPI to JS directly, but all JS-observable realm operations are faithfully exposed.

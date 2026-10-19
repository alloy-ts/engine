//! # Realm API Documentation
//!
//! ## What is a Realm?
//!
//! In the ECMAScript specification, a **Realm Record** defines the top-level execution scope
//! holding an independent global object, global environment, intrinsic constructors (`Object`,
//! `Array`, `Promise`, ...), and loaded module records.
//!
//! `boa_engine::realm::Realm` represents this in Rust:
//!
//! ```rust
//! pub struct Realm { inner: Gc<Inner> }
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
//! ## NAPI Export & Parity
//!
//! Because `boa_engine::Realm` is `!Send` and `!Sync` and holds GC-managed Rust state, it cannot
//! live directly inside a NAPI class. Instead, JS receives a serializable `Realm` handle (`{ id, ctxId }`),
//! backed by a thread-local registry (`REALM_REGISTRY`).
//!
//! ### Exported NAPI API
//!
//! - `Context.prototype.createRealm()` -> `Realm` handle
//! - `Context.prototype.enterRealm(handle)` -> Swap active realm, return previous handle
//! - `Context.prototype.realm()` -> Currently active `Realm` handle
//! - `realmRegisterProperty(realm, key, value, writable?, enumerable?, configurable?)` -> Register property on realm global
//! - `realmGlobalObject(realm)` -> JSON view of realm global object
//! - `realmGlobalThis(realm)` -> JSON view of realm `globalThis`
//! - `realmGlobalKeys(realm)` -> Keys of realm global object
//! - `realmEval(realm, source)` -> Evaluate script inside realm
//! - `realmRunJobs(realm)` -> Drain queued microtasks inside realm

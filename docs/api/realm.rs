//! # Realm Implementation API in Boa Engine and NAPI (JS) Bindings
//!
//! ## What is a Realm Record in ECMAScript & Boa Engine?
//!
//! According to the ECMAScript 2024 specification (section 9.3 Realm Records),
//! a **Realm Record** defines the execution context for code. It contains:
//!
//! - `[[Intrinsics]]`: The standard built-in objects (e.g. `%Object%`, `%Array%`, `%Promise%`).
//! - `[[GlobalObject]]`: The global object for the realm (`globalThis`).
//! - `[[GlobalEnv]]`: The global declarative environment.
//! - `[[HostDefined]]`: Field reserved for host-specific data per realm.
//! - `[[TemplateMap]]`: Cache for template literals created in this realm.
//! - `[[LoadedModules]]`: Table of loaded ES modules in this realm.
//!
//! In `boa_engine::realm::Realm`:
//! ```rust
//! pub struct Realm {
//!     inner: Gc<Inner>,
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
//! ## NAPI (JS) Exporting Strategy
//!
//! `boa_engine::realm::Realm` is `!Send` and `!Sync` and holds GC pointers (`Gc<Inner>`).
//! NAPI classes in `@napi-rs/cli` must be `Send + Sync`. Therefore:
//!
//! 1. The native `BoaRealm` lives in a `thread_local` registry (`REALM_REGISTRY`)
//!    keyed by a numeric handle `id`.
//! 2. The JS binding is exported as a serializable `#[napi(object)]` struct:
//!    ```rust
//!    #[napi(object)]
//!    pub struct Realm {
//!        pub id: u32,
//!        pub ctx_id: u32,
//!    }
//!    ```
//! 3. Realm operations are provided as NAPI free functions:
//!    - `realmRegisterProperty(realm, key, value, writable, enumerable, configurable)`
//!    - `realmGlobalObject(realm)`
//!    - `realmGlobalThis(realm)`
//!    - `realmGlobalKeys(realm)`
//!    - `realmEval(realm, source)`
//!    - `realmRunJobs(realm)`
//!
//! Every operation re-enters the realm on its owning `Context` (using `ctx_id`),
//! executes the operation, and restores the previously active realm afterwards.

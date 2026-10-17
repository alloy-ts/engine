//! # Boa Engine Realm API & NAPI (JS) Bindings
//!
//! ## What is a Realm in `boa_engine`?
//!
//! In ECMAScript (ECMA-262), a **Realm Record** encapsulates a distinct environment
//! in which code is executed. It holds:
//! - Intrinsics: built-in constructor objects and prototypes (`%Object%`, `%Array%`, `%Function%`, `%Promise%`, etc.)
//! - Global Object: the global object (`globalThis` / `window`)
//! - Global This: `globalThis` binding object
//! - Global Declarative Environment: holds top-level `let`, `const`, and `class` bindings
//! - Global Scope: lexically drives binding counts and scope structure
//! - Template Map: per-realm site-keyed cache for tagged/untagged template literals
//! - Loaded Modules: per-realm module map
//! - Host Classes: registered native/host class structures
//! - Host Defined: `[[HostDefined]]` slot for host data
//!
//! In `boa_engine`, `boa_engine::realm::Realm` is implemented as:
//!
//! ```rust
//! use std::any::TypeId;
//! use crate::{
//!     Context, HostDefined, JsNativeError, JsObject, JsResult, JsString,
//!     class::Class,
//!     context::{
//!         HostHooks,
//!         intrinsics::{Intrinsics, StandardConstructor},
//!     },
//!     environments::DeclarativeEnvironment,
//!     module::Module,
//!     object::shape::RootShape,
//! };
//! use boa_ast::scope::Scope;
//! use boa_engine::JsValue;
//! use boa_engine::property::{Attribute, PropertyDescriptor, PropertyKey};
//! use boa_gc::{Finalize, Gc, GcRef, GcRefCell, GcRefMut, Trace};
//! use rustc_hash::FxHashMap;
//!
//! /// Representation of a Realm.
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
//! #[derive(Trace, Finalize)]
//! struct Inner {
//!     intrinsics: Intrinsics,
//!     environment: Gc<DeclarativeEnvironment>,
//!     #[unsafe_ignore_trace]
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
//! ## Are we faithfully exporting the original implementation?
//!
//! **Yes.**
//!
//! Because `boa_engine::realm::Realm` is `!Send`/`!Sync` and GC-managed by `boa_gc::Gc`,
//! it cannot cross the NAPI boundary as a raw Rust reference. Instead:
//!
//! 1. **Handle Representation**: We export `Realm` as a `#[napi(object)]` struct `{ id: u32, ctx_id: u32 }`.
//!    The underlying `boa_engine::realm::Realm` is stored in a thread-local registry (`REALM_REGISTRY`).
//! 2. **Operations**: JS functions (`realmRegisterProperty`, `realmGlobalObject`, `realmGlobalThis`,
//!    `realmGlobalKeys`, `realmEval`, `realmRunJobs`) re-enter the realm on the target `Context`, run the op,
//!    and restore the previously active realm afterwards.
//! 3. **Semantics**: `realmRegisterProperty` uses `define_property_or_throw` (throwing on non-configurable collisions),
//!    faithfully matching `boa_engine::realm::Realm::register_property`.

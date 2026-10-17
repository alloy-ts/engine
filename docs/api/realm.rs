//! # Realm documentation
//!
//! `boa_engine::realm::Realm` is the Rust embodiment of the ECMAScript
//! Realm Record. It bundles a realm's intrinsics (`%Object%`, `%Array%`, ...),
//! its global declarative environment / scope, its global object and `globalThis`,
//! and per-realm caches.
//!
//! ## boa `Realm` implementation API
//!
//! ```rust
//! use std::any::TypeId;
//! use crate::{
//!     Context, HostDefined, JsNativeError, JsObject, JsResult, JsString,
//!     class::Class,
//!     context::{HostHooks, intrinsics::{Intrinsics, StandardConstructor}},
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
//! #[derive(Clone, Trace, Finalize)]
//! pub struct Realm {
//!     inner: Gc<Inner>,
//! }
//! ```
//!
//! ## NAPI (JS) Bindings
//!
//! Exposed as `#[napi(object)]` `Realm` handle:
//! ```ts
//! interface Realm {
//!   id: number;
//!   ctxId: number;
//! }
//! ```
//!
//! Operations provided as free functions (`realmRegisterProperty`, `realmGlobalObject`,
//! `realmGlobalThis`, `realmGlobalKeys`, `realmEval`, `realmRunJobs`) or methods on
//! `Context` (`createRealm`, `enterRealm`, `realm`).

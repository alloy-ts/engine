//! # Realm API Documentation
//!
//! This document details the ECMAScript `Realm` implementation in `boa_engine::realm::Realm`
//! and our JS-facing NAPI bindings exported via `@alloy-ts/engine`.
//!
//! ## What is a Realm in ECMAScript & Boa Engine?
//!
//! In the ECMAScript Specification (Table 84: Realm Record Fields), a **Realm Record**
//! represents a distinct code execution environment. Every Realm has its own intrinsic objects
//! (such as `Object`, `Array`, `Function`, `Promise`), its own global object (`globalThis`), and
//! its own global declarative environment.
//!
//! In `boa_engine`, `boa_engine::realm::Realm` is the Rust representation of an ECMAScript Realm Record.
//! Structurally, `Realm` wraps a garbage-collected pointer `boa_gc::Gc<Inner>` containing:
//!
//! ```text
//! pub struct Realm {
//!     inner: Gc<Inner>,
//! }
//!
//! struct Inner {
//!     intrinsics: Intrinsics,                           // Standard ECMAScript built-ins
//!     environment: Gc<DeclarativeEnvironment>,         // Global declarative environment
//!     scope: Scope,                                    // Global AST scope
//!     global_object: JsObject,                         // Realm global object
//!     global_this: JsObject,                           // Realm globalThis binding
//!     template_map: GcRefCell<FxHashMap<u64, JsObject>>, // Template literal cache
//!     loaded_modules: GcRefCell<FxHashMap<JsString, Module>>, // Loaded ES modules
//!     host_classes: GcRefCell<FxHashMap<TypeId, StandardConstructor>>, // Host classes
//!     host_defined: GcRefCell<HostDefined>,             // Custom host-defined data slot
//! }
//! ```
//!
//! ### `boa_engine::realm::Realm` Rust API
//!
//! | Method | Signature | Description |
//! | --- | --- | --- |
//! | `create` | `create(hooks: &dyn HostHooks, root_shape: &RootShape) -> JsResult<Self>` | Creates a new uninitialized `Realm`. |
//! | `intrinsics` | `intrinsics(&self) -> &Intrinsics` | Gets references to standard intrinsic constructors and prototypes. |
//! | `host_defined` | `host_defined(&self) -> GcRef<'_, HostDefined>` | Immutable access to host-defined data. |
//! | `host_defined_mut` | `host_defined_mut(&self) -> GcRefMut<'_, HostDefined>` | Mutable access to host-defined data. |
//! | `has_class<C>` | `has_class<C: Class>(&self) -> bool` | Checks if Rust class `C` is registered in this realm. |
//! | `get_class<C>` | `get_class<C: Class>(&self) -> Option<StandardConstructor>` | Gets constructor & prototype of class `C`. |
//! | `register_class<C>` | `register_class<C: Class>(&self, spec: StandardConstructor)` | Registers class `C` in the realm class map. |
//! | `unregister_class<C>`| `unregister_class<C: Class>(&self) -> Option<StandardConstructor>`| Removes class `C` from the realm class map. |
//! | `register_property` | `register_property(&self, key, value, attr, ctx) -> JsResult<()>` | Defines a property on the realm's global object. |
//! | `scope` | `scope(&self) -> &Scope` | Internal: Global AST scope reference. |
//! | `global_object` | `global_object(&self) -> &JsObject` | Internal: Realm global object reference. |
//! | `global_this` | `global_this(&self) -> &JsObject` | Internal: Realm `globalThis` object reference. |
//! | `addr` | `addr(&self) -> *const ()` | Internal: Returns raw pointer address of inner `Gc<Inner>`. |
//!
//! ---
//!
//! ## Are We Faithfully Exporting the Original Implementation?
//!
//! Yes, with necessary adaptations for the N-API / JavaScript boundary:
//!
//! 1. **Thread Safety & GC Ownership**:
//!    `boa_engine::realm::Realm` uses `boa_gc::Gc`, which is `!Send` and `!Sync`. It cannot cross thread or NAPI boundaries directly.
//!    We faithfully preserve Realm state by storing `boa_engine::realm::Realm` instances in a thread-local main-thread registry (`REALM_REGISTRY`) and exposing a serializable handle `{ id: u32, ctxId: u32 }` to JavaScript (`#[napi(object)] struct Realm`).
//!
//! 2. **Pointer Equality & Handle Stability**:
//!    `insert_realm` checks `Gc::ptr_eq` against existing registered realms in `REALM_REGISTRY`. Obtaining or re-entering the active realm (`ctx.realm()`, `ctx.enterRealm(handle)`) preserves handle ID identity across JS evaluations.
//!
//! 3. **Re-entrant Execution**:
//!    Realm operations (`realmEval`, `realmRegisterGlobalProperty`, `realmGlobalKeys`, `realmRunJobs`) temporarily re-enter the target realm on its parent `Context`, perform the evaluation/action in that realm, and automatically restore the previously active realm afterwards.
//!
//! 4. **Rust Generic Omissions**:
//!    Methods dependent on Rust compile-time traits/generics (`has_class<C>`, `register_class<C>`, `TypeId`) or internal AST types (`Scope`, `Intrinsics`) cannot cross into dynamic JavaScript and are omitted from JS NAPI bindings.
//!
//! ---
//!
//! ## NAPI (JS) Bindings API Reference
//!
//! ### `Realm` Handle (`#[napi(object)]`)
//!
//! ```ts
//! interface Realm {
//!   id: number;     // Opaque numeric ID in thread-local REALM_REGISTRY
//!   ctxId: number;  // ID of parent Context owning this realm
//! }
//! ```
//!
//! ### Context Realm Methods
//!
//! ```ts
//! class Context {
//!   createRealm(): Realm;           // Create new Realm initialized with default globals
//!   enterRealm(handle: Realm): Realm;// Replace active realm, returning previous handle
//!   realm(): Realm;                 // Get handle for currently active realm
//! }
//! ```
//!
//! ### Free Functions for Realm Operations
//!
//! ```ts
//! function realmEval(realm: Realm, source: string): JsonValue;
//! function realmRegisterGlobalProperty(
//!   realm: Realm,
//!   key: string,
//!   value: JsonValue,
//!   writable?: boolean,
//!   enumerable?: boolean,
//!   configurable?: boolean
//! ): void;
//! function realmGlobalKeys(realm: Realm): string[];
//! function realmRunJobs(realm: Realm): void;
//! ```
//!
//! ### Example Usage
//!
//! ```ts
//! import { Context, realmEval, realmRegisterGlobalProperty, realmGlobalKeys } from "@alloy-ts/engine";

//! const ctx = new Context();
//! const realm2 = ctx.createRealm();

//! // Register global in secondary realm
//! realmRegisterGlobalProperty(realm2, "secret", 42);

//! // Evaluate code in secondary realm
//! const val = realmEval(realm2, "secret * 2"); // 84

//! // Inspect global keys of secondary realm
//! const keys = realmGlobalKeys(realm2); // includes "secret"

//! // Switch active realm
//! const prevRealm = ctx.enterRealm(realm2);
//! ctx.eval("secret"); // 42
//! ctx.enterRealm(prevRealm); // restore original realm
//! ```

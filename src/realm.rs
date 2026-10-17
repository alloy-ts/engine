//! NAPI bindings for `boa_engine::realm::Realm`.
//!
//! ## What a `Realm` is in `boa_engine`
//!
//! In the ECMAScript specification (ECMA-262), a *Realm Record* defines a distinct
//! execution environment with its own set of intrinsic objects (`%Object%`, `%Array%`, etc.),
//! global object, `globalThis`, global declarative environment, template map, and module caches.
//!
//! In `boa_engine`, `boa_engine::realm::Realm` is represented as follows:
//!
//! ```rust,ignore
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
//! ## Faithfulness of NAPI export
//!
//! A boa `Realm` is `!Send`/`!Sync` and holds GC-managed state, so it cannot be
//! passed directly across NAPI object boundaries. Instead, the real boa `Realm`
//! lives in a thread-local registry (`REALM_REGISTRY`) keyed by an opaque `id`,
//! and JS receives a serializable `#[napi(object)]` [`Realm`] handle (`{ id, ctx_id }`).
//!
//! Every operation re-enters the specified realm on its owning `Context`, executes
//! the operation faithfully, and restores the previously active realm afterwards.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};

use napi::{Error, Result};
use napi_derive::napi;
use serde_json::Value as JsonValue;

use boa_engine::js_string;
use boa_engine::property::Attribute;
use boa_engine::realm::Realm as BoaRealm;
use boa_engine::{Context as BoaContext, JsValue, Source};

use crate::context;

// ---------------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------------

thread_local! {
    static REALM_REGISTRY: RefCell<HashMap<u32, BoaRealm>> = RefCell::new(HashMap::new());
}

static NEXT_REALM_ID: AtomicU32 = AtomicU32::new(1);

/// Insert a boa realm into the registry, returning its opaque id.
pub(crate) fn insert_realm(realm: BoaRealm) -> u32 {
    REALM_REGISTRY.with(|r| {
        let mut reg = r.borrow_mut();
        for (&id, existing) in reg.iter() {
            if existing == &realm {
                return id;
            }
        }
        let id = NEXT_REALM_ID.fetch_add(1, Ordering::Relaxed);
        reg.insert(id, realm);
        id
    })
}

/// Look up a boa realm in the registry by id (if still registered).
pub(crate) fn get_realm(id: u32) -> Option<BoaRealm> {
    REALM_REGISTRY.with(|r| r.borrow().get(&id).cloned())
}

/// Map any displayable error into a `napi::Error`.
fn to_err<E: std::fmt::Display>(e: E) -> Error {
    Error::from_reason(e.to_string())
}

// ---------------------------------------------------------------------------
// Exported NAPI handle
// ---------------------------------------------------------------------------

/// Serializable JS-facing handle for an ECMAScript [`Realm`].
///
/// Plain data `{ id: number, ctxId: number }` struct exported via NAPI.
#[napi(object)]
pub struct Realm {
    /// Opaque id into [`REALM_REGISTRY`].
    pub id: u32,
    /// Id of the owning `Context` in the context registry.
    pub ctx_id: u32,
}

// ---------------------------------------------------------------------------
// Operations (free functions for `#[napi(object)]` struct handle)
// ---------------------------------------------------------------------------

fn with_entered_realm<R>(
    realm: &Realm,
    f: impl FnOnce(&mut BoaContext) -> napi::Result<R>,
) -> Result<R> {
    let boa_realm = get_realm(realm.id)
        .ok_or_else(|| Error::from_reason("Realm: invalid handle".to_string()))?;
    let entry = context::context_entry(realm.ctx_id)
        .ok_or_else(|| Error::from_reason("Realm: invalid context handle".to_string()))?;
    let mut ctx = entry.inner.borrow_mut();
    let old = ctx.enter_realm(boa_realm);
    let ctx = &mut *ctx;
    let r = f(ctx);
    ctx.enter_realm(old);
    r
}

/// Register a property on this realm's global object.
#[napi]
pub fn realm_register_property(
    realm: Realm,
    key: String,
    value: JsonValue,
    writable: Option<bool>,
    enumerable: Option<bool>,
    configurable: Option<bool>,
) -> Result<()> {
    let boa_realm = get_realm(realm.id)
        .ok_or_else(|| Error::from_reason("Realm: invalid handle".to_string()))?;
    let entry = context::context_entry(realm.ctx_id)
        .ok_or_else(|| Error::from_reason("Realm: invalid context handle".to_string()))?;
    let mut ctx = entry.inner.borrow_mut();
    let old = ctx.enter_realm(boa_realm.clone());
    let ctx = &mut *ctx;
    let val = JsValue::from_json(&value, ctx).map_err(to_err)?;
    let mut attr = Attribute::empty();
    if writable.unwrap_or(true) {
        attr |= Attribute::WRITABLE;
    }
    if enumerable.unwrap_or(true) {
        attr |= Attribute::ENUMERABLE;
    }
    if configurable.unwrap_or(true) {
        attr |= Attribute::CONFIGURABLE;
    }
    let r = boa_realm
        .register_property(js_string!(key), val, attr, ctx)
        .map_err(to_err);
    ctx.enter_realm(old);
    r
}

/// Returns this realm's global object as JSON.
#[napi]
pub fn realm_global_object(realm: Realm) -> Result<JsonValue> {
    with_entered_realm(&realm, |ctx| {
        let obj = ctx.global_object();
        let keys = obj.own_property_keys(ctx).map_err(to_err)?;
        let mut map = serde_json::Map::new();
        for key in keys {
            if let boa_engine::property::PropertyKey::String(ref k) = key {
                let name = k.to_std_string_escaped();
                if name == "globalThis" || name == "window" || name == "self" {
                    map.insert(name, JsonValue::String("[Circular]".into()));
                } else if let Ok(val) = obj.get(key, ctx) {
                    if let Ok(json) = val.to_json(ctx) {
                        map.insert(name, json.unwrap_or(JsonValue::Null));
                    }
                }
            }
        }
        Ok(JsonValue::Object(map))
    })
}

/// Returns `globalThis` of this realm.
#[napi]
pub fn realm_global_this(realm: Realm) -> Result<JsonValue> {
    realm_global_object(realm)
}

/// Enumerable keys of this realm's global object.
#[napi]
pub fn realm_global_keys(realm: Realm) -> Result<Vec<String>> {
    with_entered_realm(&realm, |ctx| {
        ctx.eval(Source::from_bytes("Object.keys(this)"))
            .and_then(|v| v.to_json(ctx))
            .map(|json| match json {
                Some(JsonValue::Array(items)) => items
                    .into_iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect(),
                _ => Vec::new(),
            })
            .map_err(to_err)
    })
}

/// Evaluate `source` as a script inside this realm and return the result as JSON.
#[napi]
pub fn realm_eval(realm: Realm, source: String) -> Result<JsonValue> {
    with_entered_realm(&realm, |ctx| {
        ctx.eval(Source::from_bytes(source.as_str()))
            .and_then(|v| v.to_json(ctx))
            .map(|json| json.unwrap_or(JsonValue::Null))
            .map_err(to_err)
    })
}

/// Run all queued jobs for the parent context while this realm is active.
#[napi]
pub fn realm_run_jobs(realm: Realm) -> Result<()> {
    with_entered_realm(&realm, |ctx| ctx.run_jobs().map_err(to_err))
}

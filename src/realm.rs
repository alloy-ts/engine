//! NAPI bindings for `boa_engine::realm::Realm`.
//!
//! ## What a `Realm` is
//!
//! In the ECMAScript specification a *Realm Record* bundles together a realm's
//! intrinsics (the `%Object%`, `%Array%`, ... constructors), its global
//! declarative environment / scope, its global object and `globalThis`, and the
//! per-realm caches (template literals, loaded modules, host-defined data,
//! registered host classes). `boa_engine::realm::Realm` is the Rust embodiment:
//! a cheaply-cloneable, GC-managed handle (`Gc<Inner>`) whose equality is pointer
//! identity of the inner record.
//!
//! A boa `Realm` is `!Send`/`!Sync` and holds GC-managed state, so it cannot be
//! stored directly inside an napi object. Instead the real boa `Realm` lives in a
//! thread-local registry keyed by an opaque `id`, and JS sees a **serializable**
//! `#[napi(object)]` [`Realm`] handle (`{ id, ctxId }`). Every realm operation
//! re-enters the realm on its owning `Context` to run the op, restoring the
//! previously-active realm afterwards.
//!
//! Realms are created via `Context.createRealm()` / `Context.enterRealm()` /
//! `Context.realm()` (see `src/context.rs`); the functions in this module operate
//! on the resulting handle.
//!
//! See `docs/api/realm.md` for the full boa-API → NAPI parity table.

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

// Process-wide, **main-thread** registry that owns every boa [`Realm`].
//
// A realm is created inside a specific boa `Context`; we store the boa `Realm`
// here (rooted by the `Gc` pointer) so it stays valid as long as its JS handle
// and the parent `Context` are alive. Like the `Context` registry this is
// `thread_local` (boa realms belong to the thread their context was created on).
// **Limitation:** main-thread only.
thread_local! {
    static REALM_REGISTRY: RefCell<HashMap<u32, BoaRealm>> = RefCell::new(HashMap::new());
}

/// Monotonic id source. `0` is reserved as the "invalid handle" sentinel.
static NEXT_REALM_ID: AtomicU32 = AtomicU32::new(1);

/// Insert a boa realm into the registry, returning its opaque id.
pub(crate) fn insert_realm(realm: BoaRealm) -> u32 {
    let id = NEXT_REALM_ID.fetch_add(1, Ordering::Relaxed);
    REALM_REGISTRY.with(|r| r.borrow_mut().insert(id, realm));
    id
}

/// Look up a boa realm in the registry by id (if still registered).
pub(crate) fn get_realm(id: u32) -> Option<BoaRealm> {
    REALM_REGISTRY.with(|r| r.borrow().get(&id).cloned())
}

/// Map any displayable error (boa's `JsError`, etc.) into a `napi::Error`.
fn to_err<E: std::fmt::Display>(e: E) -> Error {
    Error::from_reason(e.to_string())
}

// ---------------------------------------------------------------------------
// Exported handle
// ---------------------------------------------------------------------------

/// Serializable, JS-facing handle for an ECMAScript [`Realm`][boa].
///
/// This is a `#[napi(object)]` (plain data), not a `#[napi]` class: a realm
/// wraps a `boa_engine::Realm`, which is `!Send`/`!Sync` and holds non-napi,
/// GC-managed fields, so it cannot live directly inside an napi object. The real
/// boa `Realm` lives in [`REALM_REGISTRY`]; this carries the opaque `id` plus the
/// `ctxId` of the owning `Context` so operations can re-enter it.
///
/// Created via `Context.createRealm()` / `Context.enterRealm()` /
/// `Context.realm()` and consumed by the `realm*` free functions below.
///
/// ```ts
/// const ctx = new Context();
/// const realm = ctx.createRealm();
/// realm; // { id: number, ctxId: number }
/// ```
#[napi(object)]
pub struct Realm {
    /// Opaque id into [`REALM_REGISTRY`].
    pub id: u32,
    /// Id of the owning `Context` in the context registry.
    pub ctx_id: u32,
}

// ---------------------------------------------------------------------------
// Operations (free functions — `#[napi(object)]` structs cannot carry methods)
//
// Naming mirrors the boa `Realm` API where one exists:
//   `Realm::register_property`  -> `realmRegisterProperty`
//   `Realm::global_object`      -> `realmGlobalObject`
//   `Realm::global_this`        -> `realmGlobalThis`
// `realmEval` / `realmGlobalKeys` / `realmRunJobs` are conveniences that map to
// the owning `Context`'s `eval` / `global_keys` / `run_jobs` while the realm is
// active (boa's `Realm` has no such methods — see docs/api/realm.md).
// ---------------------------------------------------------------------------

/// Run `f` with the boa `Context` stored under `realm.ctx_id` borrowed mutably
/// (if the context is still registered), after re-entering `realm`, restoring
/// the previously-active realm afterwards.
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

/// Faithful mapping of [`boa_engine::realm::Realm::register_property`].
///
/// Registers a property on this realm's global object. boa uses
/// `define_property_or_throw`, so the call (re)defines the property when an
/// existing one is configurable, and throws a `TypeError` when a
/// **non-configurable** property of the same name already exists.
///
/// `writable` / `enumerable` / `configurable` default to `true` when omitted.
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

/// Faithful mapping of [`boa_engine::realm::Realm::global_object`] (JSON form).
///
/// Returns this realm's global object, JSON-marshalled (functions / symbols /
/// promises collapse to `null`).
#[napi]
pub fn realm_global_object(realm: Realm) -> Result<JsonValue> {
    with_entered_realm(&realm, |ctx| {
        ctx.eval(Source::from_bytes(
            b"(() => { const res = {}; for (const k of Object.getOwnPropertyNames(globalThis)) { try { const val = globalThis[k]; if (val === globalThis) continue; const t = typeof val; if (t === 'number' || t === 'string' || t === 'boolean' || val === null) { res[k] = val; } else if (t === 'object') { try { const str = JSON.stringify(val); if (str !== undefined) { res[k] = JSON.parse(str); } } catch (_) {} } } catch (_) {} } return res; })()"
        ))
            .and_then(|v| v.to_json(ctx))
            .map(|json| json.unwrap_or(JsonValue::Null))
            .map_err(to_err)
    })
}

/// Faithful mapping of [`boa_engine::realm::Realm::global_this`] (JSON form).
///
/// Returns `globalThis` of this realm. Under boa's `DefaultHooks` this is the
/// same object as [`realm_global_object`]; custom `HostHooks` may diverge, but
/// boa's `global_this()` is `pub(crate)` so the binding reads it via the realm's
/// own `this`.
#[napi]
pub fn realm_global_this(realm: Realm) -> Result<JsonValue> {
    with_entered_realm(&realm, |ctx| {
        ctx.eval(Source::from_bytes(
            b"(() => { const res = {}; for (const k of Object.getOwnPropertyNames(this)) { try { const val = this[k]; if (val === this) continue; const t = typeof val; if (t === 'number' || t === 'string' || t === 'boolean' || val === null) { res[k] = val; } else if (t === 'object') { try { const str = JSON.stringify(val); if (str !== undefined) { res[k] = JSON.parse(str); } } catch (_) {} } } catch (_) {} } return res; })()"
        ))
            .and_then(|v| v.to_json(ctx))
            .map(|json| json.unwrap_or(JsonValue::Null))
            .map_err(to_err)
    })
}

/// Enumerable keys of this realm's global object (`Object.keys(this)`).
///
/// Convenience — boa's `Realm` has no such method; this reads the realm's own
/// global object after re-entering it.
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

/// Evaluate `source` as a script **inside this realm** and return the result as
/// JSON. Does not auto-run jobs — call [`realm_run_jobs`] after, like native boa.
///
/// Convenience — maps to the owning `Context::eval` while the realm is active
/// (boa's `Realm` itself has no `eval`).
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
///
/// Convenience — maps to the owning `Context::run_jobs`.
#[napi]
pub fn realm_run_jobs(realm: Realm) -> Result<()> {
    with_entered_realm(&realm, |ctx| ctx.run_jobs().map_err(to_err))
}

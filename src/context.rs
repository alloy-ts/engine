//! JS/TS bindings for `boa_engine::Context`.
//!
//! See `docs/api/context.md` (the "JS/TS API (napi bindings) contract" section)
//! for the full JS-facing contract.
//!
//! # Ownership design
//!
//! `boa_engine::Context` is `!Send`/`!Sync`, so it **cannot** be a field of a
//! `#[napi]` class. Each JS `Context` therefore holds only an opaque `u32` id;
//! the real context lives in a thread-local `RefCell` registry and is borrowed
//! mutably on every call (boa's entry points need `&mut Context` while N-API
//! only hands us `&self` — serialization is guaranteed by the single-threaded
//! JS environment, which is also all boa allows).
//!
//! A second registry holds `Realm` handles, and a per-context keyed store backs
//! the host-defined data API.

use std::cell::RefCell;
use std::collections::HashMap;
use std::ptr;
use std::ptr::NonNull;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

use napi::{Env, Error, JsValue as NapiJsValue, NapiRaw, sys};
use napi::bindgen_prelude::{FromNapiValue, ToNapiValue};
use napi::check_status;
use napi_derive::napi;
use rquickjs::context::intrinsic;
use rquickjs::{qjs, Ctx, FromJs, Module, Runtime, Context as Inner, Type, Value};
use serde_json::Value as JsonValue;

use crate::engine::JsEngine;

// ---------------------------------------------------------------------------
// QuickJS-backed context (`QjsContext` / `JsContext`) — retained for `Atom` and
// the QuickJS `Engine`. This is a rename of the previous `Context` / `JsCtx` so
// `Atom` (which is intrinsically QuickJS) keeps working under napi 3 types.
// ---------------------------------------------------------------------------

static MODULE_SEQ: AtomicU32 = AtomicU32::new(0);

/// Map a [`rquickjs::Error`] into a `napi::Error` so JS sees a thrown exception.
pub(crate) fn qjs_err(e: rquickjs::Error) -> Error {
    Error::from_reason(e.to_string())
}

/// Convert a QuickJS value into a JSON value Node can receive directly.
pub(crate) fn qjs_value_to_json<'js>(ctx: &Ctx<'js>, value: Value<'js>) -> napi::Result<JsonValue> {
    use rquickjs::String as QjsString;

    match value.type_of() {
        Type::Undefined | Type::Null | Type::Uninitialized => Ok(JsonValue::Null),
        Type::Bool => Ok(JsonValue::Bool(bool::from_js(ctx, value).map_err(qjs_err)?)),
        Type::Int => Ok(JsonValue::Number(
            i64::from_js(ctx, value).map_err(qjs_err)?.into(),
        )),
        Type::Float => {
            let f = f64::from_js(ctx, value).map_err(qjs_err)?;
            Ok(serde_json::Number::from_f64(f)
                .map(JsonValue::Number)
                .unwrap_or(JsonValue::Null))
        }
        Type::String => Ok(JsonValue::String(
            QjsString::from_js(ctx, value)
                .map_err(qjs_err)?
                .to_string()
                .map_err(qjs_err)?,
        )),
        _ => match ctx.json_stringify(value).map_err(qjs_err)? {
            Some(s) => serde_json::from_str(&s.to_string().map_err(qjs_err)?)
                .map_err(|e| Error::from_reason(e.to_string())),
            None => Ok(JsonValue::Null),
        },
    }
}

/// NAPI class wrapping an embedded QuickJS [`Context`](rquickjs::Context).
#[napi]
pub struct QjsContext {
    inner: Inner,
    env: Env,
    alive: Arc<AtomicBool>,
}

#[napi]
impl QjsContext {
    #[napi(constructor)]
    pub fn new(env: Env) -> napi::Result<Self> {
        let runtime = Runtime::new().map_err(qjs_err)?;
        let inner = Inner::full(&runtime).map_err(qjs_err)?;
        Self::create_standalone(inner, env)
    }

    pub(crate) fn from_inner(inner: Inner, env: Env, alive: Arc<AtomicBool>) -> Self {
        QjsContext { inner, env, alive }
    }

    fn create_standalone(inner: Inner, env: Env) -> napi::Result<Self> {
        let alive = Arc::new(AtomicBool::new(false));
        let hook_data = Arc::clone(&alive);
        env.add_env_cleanup_hook(hook_data, |flag: Arc<AtomicBool>| {
            flag.store(true, Ordering::SeqCst);
        })?;
        Ok(QjsContext { inner, env, alive })
    }

    pub(crate) fn clone_inner(&self) -> Inner {
        self.inner.clone()
    }

    #[napi]
    pub fn eval(&self, source: String) -> napi::Result<JsonValue> {
        self.inner.with(|ctx| {
            let value: Value = ctx.eval(source).map_err(qjs_err)?;
            qjs_value_to_json(&ctx, value)
        })
    }

    #[napi]
    pub fn eval_module(&self, source: String) -> napi::Result<JsonValue> {
        self.inner.with(|ctx| {
            let seq = MODULE_SEQ.fetch_add(1, Ordering::Relaxed);
            let name = format!("<eval:{seq}>");
            let module = Module::declare(ctx.clone(), name, source).map_err(qjs_err)?;
            let (evaluated, _promise) = module.eval().map_err(qjs_err)?;
            let ns = evaluated.namespace().map_err(qjs_err)?;
            qjs_value_to_json(&ctx, ns.into_value())
        })
    }

    #[napi]
    pub fn json_parse(&self, json: String) -> napi::Result<JsonValue> {
        self.inner.with(|ctx| {
            let value = ctx.json_parse(json).map_err(qjs_err)?;
            qjs_value_to_json(&ctx, value)
        })
    }

    #[napi]
    pub fn global_keys(&self) -> napi::Result<Vec<String>> {
        self.inner.with(|ctx| {
            let keys: Vec<String> = ctx.eval("Object.keys(this)").map_err(qjs_err)?;
            Ok(keys)
        })
    }

    #[napi]
    pub fn run_gc(&self) -> napi::Result<()> {
        self.inner.with(|ctx| ctx.run_gc());
        Ok(())
    }

    #[napi]
    pub fn execute_pending_job(&self) -> napi::Result<bool> {
        Ok(self.inner.with(|ctx| ctx.execute_pending_job()))
    }

    #[napi]
    pub fn has_exception(&self) -> napi::Result<bool> {
        Ok(self.inner.with(|ctx| ctx.has_exception()))
    }

    #[napi]
    pub fn catch(&self) -> napi::Result<JsonValue> {
        self.inner.with(|ctx| {
            let value = ctx.catch();
            qjs_value_to_json(&ctx, value)
        })
    }

    #[napi]
    pub fn full(env: Env) -> napi::Result<Self> {
        let runtime = Runtime::new().map_err(qjs_err)?;
        let inner = Inner::full(&runtime).map_err(qjs_err)?;
        Self::create_standalone(inner, env)
    }

    #[napi]
    pub fn base(env: Env) -> napi::Result<Self> {
        let runtime = Runtime::new().map_err(qjs_err)?;
        let inner = Inner::base(&runtime).map_err(qjs_err)?;
        Self::create_standalone(inner, env)
    }

    #[napi]
    pub fn custom(env: Env, intrinsics: Option<Vec<String>>) -> napi::Result<Self> {
        let runtime = Runtime::new().map_err(qjs_err)?;
        let use_full = intrinsics
            .map_or(false, |list| list.iter().any(|s| s == "All"));
        let inner = if use_full {
            Inner::full(&runtime)
        } else {
            Inner::custom::<intrinsic::None>(&runtime)
        }
        .map_err(qjs_err)?;
        Self::create_standalone(inner, env)
    }

    #[napi]
    pub fn builder(env: Env) -> napi::Result<Self> {
        let runtime = Runtime::new().map_err(qjs_err)?;
        let inner = Inner::builder().build(&runtime).map_err(qjs_err)?;
        Self::create_standalone(inner, env)
    }

    #[napi]
    pub fn from_raw(env: Env, ctx_ptr: napi::bindgen_prelude::BigInt, engine: &JsEngine) -> napi::Result<Self> {
        let (lossless, value, _neg) = ctx_ptr.get_u64();
        if !lossless {
            return Err(Error::from_reason("context pointer does not fit in a u64"));
        }
        let ptr = value as *mut qjs::JSContext;
        let nn = NonNull::new(ptr)
            .ok_or_else(|| Error::from_reason("context pointer is null"))?;
        let inner = unsafe { Inner::from_raw(nn, engine.inner.clone()) };
        Self::create_standalone(inner, env)
    }

    #[napi]
    pub fn as_raw(&self) -> napi::Result<napi::bindgen_prelude::BigInt> {
        let ptr = self.inner.as_raw().as_ptr() as u64;
        Ok(napi::bindgen_prelude::BigInt::from(ptr))
    }

    #[napi]
    pub fn get_engine_ptr(&self) -> napi::Result<napi::bindgen_prelude::BigInt> {
        let ptr = self.inner.get_runtime_ptr() as u64;
        Ok(napi::bindgen_prelude::BigInt::from(ptr))
    }

    #[napi(js_name = "Engine")]
    pub fn engine(&self) -> napi::Result<JsEngine> {
        JsEngine::new(self.env, None)
    }

    #[napi]
    pub fn with(&self, callback: napi::JsObject) -> napi::Result<JsonValue> {
        self.inner.with(|ctx| {
            let ctx_static = unsafe { std::mem::transmute::<Ctx<'_>, Ctx<'static>>(ctx) };
            let jsctx = JsContext::from_inner(ctx_static);
            let jsctx_raw = unsafe { <JsContext as ToNapiValue>::to_napi_value(self.env.raw(), jsctx) }?;
            let mut return_value = ptr::null_mut();
            check_status!(
                unsafe {
                    sys::napi_call_function(
                        self.env.raw(),
                        ptr::null_mut(),
                        callback.raw(),
                        1,
                        &jsctx_raw as *const _,
                        &mut return_value,
                    )
                },
                "QjsContext.with callback failed"
            )?;
            unsafe { <JsonValue as FromNapiValue>::from_napi_value(self.env.raw(), return_value) }
        })
    }
}

/// A `Ctx` handle handed to a QuickJS `with` callback (used by `Atom`).
#[napi]
pub struct JsContext {
    inner: Ctx<'static>,
}

#[napi]
impl JsContext {
    pub(crate) fn from_inner(inner: Ctx<'static>) -> Self {
        JsContext { inner }
    }

    pub(crate) fn clone_inner(&self) -> Ctx<'static> {
        self.inner.clone()
    }

    #[napi]
    pub fn eval(&self, source: String) -> napi::Result<JsonValue> {
        let value: Value = self.inner.eval(source).map_err(qjs_err)?;
        qjs_value_to_json(&self.inner, value)
    }

    #[napi]
    pub fn eval_module(&self, source: String) -> napi::Result<JsonValue> {
        let seq = MODULE_SEQ.fetch_add(1, Ordering::Relaxed);
        let name = format!("<eval:{seq}>");
        let module = Module::declare(self.inner.clone(), name, source).map_err(qjs_err)?;
        let (evaluated, _promise) = module.eval().map_err(qjs_err)?;
        let ns = evaluated.namespace().map_err(qjs_err)?;
        qjs_value_to_json(&self.inner, ns.into_value())
    }

    #[napi]
    pub fn json_parse(&self, json: String) -> napi::Result<JsonValue> {
        let value = self.inner.json_parse(json).map_err(qjs_err)?;
        qjs_value_to_json(&self.inner, value)
    }

    #[napi]
    pub fn global_keys(&self) -> napi::Result<Vec<String>> {
        let keys: Vec<String> = self.inner.eval("Object.keys(this)").map_err(qjs_err)?;
        Ok(keys)
    }

    #[napi]
    pub fn run_gc(&self) -> napi::Result<()> {
        self.inner.run_gc();
        Ok(())
    }

    #[napi]
    pub fn execute_pending_job(&self) -> napi::Result<bool> {
        Ok(self.inner.execute_pending_job())
    }

    #[napi]
    pub fn has_exception(&self) -> napi::Result<bool> {
        Ok(self.inner.has_exception())
    }

    #[napi]
    pub fn catch(&self) -> napi::Result<JsonValue> {
        let value = self.inner.catch();
        qjs_value_to_json(&self.inner, value)
    }
}

// ---------------------------------------------------------------------------
// boa-backed `Context` — the headline JS class.
// ---------------------------------------------------------------------------

use boa_engine::{
    js_string, Context as BoaContext, JsValue, NativeFunction, Source,
};
use boa_engine::object::ObjectInitializer;
use boa_engine::optimizer::OptimizerOptions;
use boa_engine::property::Attribute;

use crate::realm::{get_realm, insert_realm, Realm};

/// Map a boa [`JsError`](boa_engine::JsError) onto a `napi::Error`.
pub(crate) fn to_napi_err(e: boa_engine::JsError) -> Error {
    Error::from_reason(e.to_string())
}

/// Convert a `napi::Error` into a boa `JsError` so it can be thrown out of a
/// native function registered into a boa `Context` (whose closures must return
/// `JsError`, not `napi::Error`).
fn napi_to_js_err(e: Error, _context: &mut BoaContext) -> boa_engine::JsError {
    boa_engine::JsError::from_opaque(js_string!(e.to_string()).into())
}

/// Marshal a boa `JsValue` into a `serde_json::Value` for JS.
///
/// Mirrors the convention used by the QuickJS path: functions, symbols and
/// promises are not JSON-representable and collapse to `null`.
pub(crate) fn jsvalue_to_json(value: &JsValue, context: &mut BoaContext) -> napi::Result<JsonValue> {
    Ok(value
        .to_json(context)
        .map_err(to_napi_err)?
        .unwrap_or(JsonValue::Null))
}

/// Convert a JS-provided `serde_json::Value` into a boa `JsValue`.
pub(crate) fn json_to_jsvalue(value: JsonValue, context: &mut BoaContext) -> napi::Result<JsValue> {
    JsValue::from_json(&value, context).map_err(to_napi_err)
}

pub(crate) struct ContextEntry {
    pub(crate) inner: RefCell<BoaContext>,
    /// Keyed reinterpretation of boa's typed `HostDefined<T>` slot store.
    pub(crate) host_data: RefCell<HashMap<String, JsValue>>,
    /// JS callbacks queued via `enqueueJob`, drained by `runJobs`.
    pub(crate) job_queue: RefCell<Vec<(Env, sys::napi_ref)>>,
}

thread_local! {
    static CONTEXTS: RefCell<HashMap<u32, Rc<ContextEntry>>> = RefCell::new(HashMap::new());
}

static NEXT_ID: AtomicU32 = AtomicU32::new(1);

pub(crate) fn alloc_id() -> u32 {
    loop {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        if id == 0 {
            continue;
        }
        let taken = CONTEXTS.with(|c| c.borrow().contains_key(&id));
        if taken {
            continue;
        }
        return id;
    }
}

fn insert_context(ctx: BoaContext) -> u32 {
    let id = alloc_id();
    CONTEXTS.with(|c| {
        c.borrow_mut().insert(
            id,
            Rc::new(ContextEntry {
                inner: RefCell::new(ctx),
                host_data: RefCell::new(HashMap::new()),
                job_queue: RefCell::new(Vec::new()),
            }),
        )
    });
    id
}

/// Look up a context entry by id (crate-internal; used by the realm module).
pub(crate) fn context_entry(id: u32) -> Option<Rc<ContextEntry>> {
    CONTEXTS.with(|c| c.borrow().get(&id).cloned())
}

/// Run `f` with a mutable borrow of the boa context behind `id`, or `None` if the
/// id is unknown or the registry is already borrowed (re-entrant call).
pub(crate) fn with_boa_context_mut<R>(id: u32, f: impl FnOnce(&mut BoaContext) -> R) -> Option<R> {
    let entry = context_entry(id)?;
    let mut ctx = entry.inner.borrow_mut();
    Some(f(&mut ctx))
}

/// Register a `console` global (`log`/`info`/`debug`/`error`/`warn`/`trace`) so
/// that `console.log(...)` works out of the box, mirroring browser/node output.
fn install_default_globals(ctx: &mut BoaContext) {
    let make = |is_err: bool| {
        NativeFunction::from_copy_closure(move |_this, args, context| {
            let parts: Vec<String> = args
                .iter()
                .filter_map(|a| a.to_string(context).ok())
                .map(|s| s.to_std_string_escaped())
                .collect();
            let line = parts.join(" ");
            if is_err {
                eprintln!("{}", line);
            } else {
                println!("{}", line);
            }
            Ok(JsValue::undefined())
        })
    };

    let console = ObjectInitializer::new(ctx)
        .function(make(false), js_string!("log"), 0)
        .function(make(false), js_string!("info"), 0)
        .function(make(false), js_string!("debug"), 0)
        .function(make(true), js_string!("error"), 0)
        .function(make(true), js_string!("warn"), 0)
        .function(make(false), js_string!("trace"), 0)
        .build();

    let _ = ctx.register_global_property(js_string!("console"), console, Attribute::all());
}

/// Build a boa `Attribute` bitmask from the JS numeric flags.
pub(crate) fn attribute_from_u32(flags: u32) -> Attribute {
    let mut a = Attribute::empty();
    if flags & 1 != 0 {
        a |= Attribute::WRITABLE;
    }
    if flags & 2 != 0 {
        a |= Attribute::ENUMERABLE;
    }
    if flags & 4 != 0 {
        a |= Attribute::CONFIGURABLE;
    }
    a
}

/// Marshal a boa `JsValue` into a raw `napi_value` (via JSON) suitable for
/// passing as an argument to a host `JsFunction`.
fn jsvalue_to_napi_raw(
    value: &JsValue,
    context: &mut BoaContext,
    env: &Env,
) -> napi::Result<sys::napi_value> {
    let json = jsvalue_to_json(value, context)?;
    unsafe { <JsonValue as ToNapiValue>::to_napi_value(env.raw(), json) }
}

fn napi_value_to_json(env_raw: sys::napi_env, val: sys::napi_value) -> napi::Result<JsonValue> {
    let mut value_type = sys::ValueType::napi_undefined;
    check_status!(unsafe { sys::napi_typeof(env_raw, val, &mut value_type) }, "napi_typeof failed")?;
    match value_type {
        sys::ValueType::napi_undefined | sys::ValueType::napi_null | sys::ValueType::napi_symbol | sys::ValueType::napi_function => Ok(JsonValue::Null),
        sys::ValueType::napi_boolean => {
            let mut b = false;
            check_status!(unsafe { sys::napi_get_value_bool(env_raw, val, &mut b) }, "napi_get_value_bool failed")?;
            Ok(JsonValue::Bool(b))
        }
        sys::ValueType::napi_number => {
            let mut num = 0.0;
            check_status!(unsafe { sys::napi_get_value_double(env_raw, val, &mut num) }, "napi_get_value_double failed")?;
            Ok(serde_json::Number::from_f64(num).map(JsonValue::Number).unwrap_or(JsonValue::Null))
        }
        sys::ValueType::napi_string => {
            let str_val = unsafe { <String as FromNapiValue>::from_napi_value(env_raw, val) }?;
            Ok(JsonValue::String(str_val))
        }
        _ => {
            let json = unsafe { <JsonValue as FromNapiValue>::from_napi_value(env_raw, val) };
            json.or_else(|_| Ok(JsonValue::Null))
        }
    }
}

#[napi(object)]
pub struct RuntimeLimitsView {
    pub stack_size: Option<u32>,
    pub loop_iteration_limit: Option<f64>,
    pub backtrace_limit: Option<u32>,
    pub recursion_limit: Option<u32>,
}

#[napi(object)]
pub struct OptimizerOptionsView {
    pub statistics: Option<bool>,
    pub constant_folding: Option<bool>,
    pub strength_reduction: Option<bool>,
    pub dead_code_elimination: Option<bool>,
}

#[napi(object)]
pub struct StackFrameView {
    pub function_name: String,
    pub line: Option<u32>,
    pub column: Option<u32>,
}

/// An ECMAScript execution context backed by [`boa_engine::Context`].
///
/// The real boa context lives in a thread-local `RefCell` registry; this JS
/// class holds only an opaque numeric id.
///
/// ```ts
/// const ctx = new Context();
/// const v = ctx.eval("1 + 1"); // 2
/// ```
#[napi]
pub struct Context {
    id: u32,
}

#[napi]
impl Context {
    /// Create a `Context` with default settings. Throws (as a JS exception) if
    /// the default build fails.
    #[napi(constructor)]
    pub fn new() -> napi::Result<Self> {
        let mut ctx = BoaContext::builder().build().map_err(to_napi_err)?;
        install_default_globals(&mut ctx);
        Ok(Context { id: insert_context(ctx) })
    }

    /// Start building a `Context` with explicit options.
    #[napi(factory)]
    pub fn builder() -> ContextBuilder {
        ContextBuilder { can_block: false }
    }

    /// Remove this context from the registry, freeing its boa state.
    #[napi]
    pub fn dispose(&self) -> napi::Result<()> {
        CONTEXTS.with(|c| c.borrow_mut().remove(&self.id));
        Ok(())
    }

    fn entry(&self) -> napi::Result<Rc<ContextEntry>> {
        CONTEXTS
            .with(|c| c.borrow().get(&self.id).cloned())
            .ok_or_else(|| Error::from_reason("Context has been disposed"))
    }

    /// Evaluate `source` as a global script; returns the completion value as
    /// JSON. Does **not** drain promise jobs — call [`Context::runJobs`] after.
    #[napi]
    pub fn eval(&self, source: String) -> napi::Result<JsonValue> {
        let entry = self.entry()?;
        let mut ctx = entry.inner.borrow_mut();
        let value = ctx
            .eval(Source::from_bytes(source.as_bytes()))
            .map_err(to_napi_err)?;
        jsvalue_to_json(&value, &mut ctx)
    }

    /// Run all queued promise jobs, then any JS callbacks queued via
    /// [`Context::enqueueJob`].
    #[napi]
    pub fn run_jobs(&self) -> napi::Result<()> {
        let entry = self.entry()?;
        {
            let mut ctx = entry.inner.borrow_mut();
            ctx.run_jobs().map_err(to_napi_err)?;
        }
        let jobs = entry.job_queue.borrow_mut().drain(..).collect::<Vec<_>>();
        for (env, raw_ref) in jobs {
            unsafe {
                let mut raw_val = ptr::null_mut();
                sys::napi_get_reference_value(env.raw(), raw_ref, &mut raw_val);
                if !raw_val.is_null() {
                    let mut undef = ptr::null_mut();
                    sys::napi_get_undefined(env.raw(), &mut undef);
                    let mut return_val = ptr::null_mut();
                    sys::napi_call_function(
                        env.raw(),
                        undef,
                        raw_val,
                        0,
                        ptr::null(),
                        &mut return_val,
                    );
                }
                sys::napi_delete_reference(env.raw(), raw_ref);
            }
        }
        Ok(())
    }

    /// Queue a JS `callback` to run on the next [`Context::runJobs`].
    #[napi]
    pub fn enqueue_job(&self, env: Env, callback: napi::JsObject) -> napi::Result<()> {
        let entry = self.entry()?;
        let mut raw_ref = ptr::null_mut();
        check_status!(
            unsafe { sys::napi_create_reference(env.raw(), callback.raw(), 1, &mut raw_ref) },
            "enqueueJob failed to create reference"
        )?;
        entry.job_queue.borrow_mut().push((env, raw_ref));
        Ok(())
    }

    /// Set the context's strict-mode flag.
    #[napi]
    pub fn strict(&self, mode: bool) -> napi::Result<()> {
        let entry = self.entry()?;
        entry.inner.borrow_mut().strict(mode);
        Ok(())
    }

    // ---- Globals & registration ----

    /// Register a global property. Errors if it already exists.
    #[napi]
    pub fn register_global_property(
        &self,
        key: String,
        value: JsonValue,
        attribute: u32,
    ) -> napi::Result<()> {
        let entry = self.entry()?;
        let mut ctx = entry.inner.borrow_mut();
        let value = json_to_jsvalue(value, &mut ctx)?;
        ctx.register_global_property(js_string!(key.as_str()), value, attribute_from_u32(attribute))
            .map_err(to_napi_err)
    }

    /// Register a global callable that is both callable and constructable.
    #[napi]
    pub fn register_global_callable(
        &self,
        env: Env,
        name: String,
        length: u32,
        callback: napi::JsObject,
    ) -> napi::Result<()> {
        self.register_callable(env, name, length, callback, true)
    }

    /// Register a global callable that is **not** a constructor (usage as a
    /// constructor yields a `TypeError`).
    #[napi]
    pub fn register_global_builtin_callable(
        &self,
        env: Env,
        name: String,
        length: u32,
        callback: napi::JsObject,
    ) -> napi::Result<()> {
        self.register_callable(env, name, length, callback, false)
    }

    fn register_callable(
        &self,
        env: Env,
        name: String,
        length: u32,
        callback: napi::JsObject,
        constructable: bool,
    ) -> napi::Result<()> {
        let entry = self.entry()?;
        let mut ctx = entry.inner.borrow_mut();
        let env_raw = env.raw();
        let mut raw_ref = ptr::null_mut();
        check_status!(
            unsafe { sys::napi_create_reference(env_raw, callback.raw(), 1, &mut raw_ref) },
            "register_callable failed to create reference"
        )?;
        let native = NativeFunction::from_copy_closure(move |_this, args, context| {
            let env = Env::from_raw(env_raw);
            let mut func_val = ptr::null_mut();
            check_status!(
                unsafe { sys::napi_get_reference_value(env_raw, raw_ref, &mut func_val) },
                "failed to dereference callback"
            )
            .map_err(|e| napi_to_js_err(e, context))?;

            let mut argv: Vec<sys::napi_value> = Vec::with_capacity(args.len());
            for a in args {
                argv.push(
                    jsvalue_to_napi_raw(a, context, &env)
                        .map_err(|e| napi_to_js_err(e, context))?,
                );
            }
            let mut return_value = ptr::null_mut();
            let mut undef = ptr::null_mut();
            unsafe { sys::napi_get_undefined(env_raw, &mut undef) };
            check_status!(
                unsafe {
                    sys::napi_call_function(
                        env_raw,
                        undef,
                        func_val,
                        argv.len(),
                        if argv.is_empty() { ptr::null() } else { argv.as_ptr() },
                        &mut return_value,
                    )
                },
                "registered global callable threw"
            )
            .map_err(|e| napi_to_js_err(e, context))?;
            let json = napi_value_to_json(env_raw, return_value)
                .map_err(|e| napi_to_js_err(e, context))?;
            JsValue::from_json(&json, context)
        });
        if constructable {
            ctx.register_global_callable(js_string!(name.as_str()), length as usize, native)
                .map_err(to_napi_err)
        } else {
            ctx.register_global_builtin_callable(js_string!(name.as_str()), length as usize, native)
                .map_err(to_napi_err)
        }
    }

    // ---- Accessors ----

    /// Enumerable own keys of the global object.
    #[napi]
    pub fn global_keys(&self) -> napi::Result<Vec<String>> {
        let entry = self.entry()?;
        let mut ctx = entry.inner.borrow_mut();
        let v = ctx
            .eval(Source::from_bytes(b"Object.keys(globalThis)"))
            .map_err(to_napi_err)?;
        match jsvalue_to_json(&v, &mut ctx)? {
            JsonValue::Array(arr) => Ok(arr
                .into_iter()
                .filter_map(|x| x.as_str().map(str::to_string))
                .collect()),
            _ => Ok(Vec::new()),
        }
    }

    /// The global object, JSON-marshalled.
    #[napi]
    pub fn global_object(&self) -> napi::Result<JsonValue> {
        let entry = self.entry()?;
        let mut ctx = entry.inner.borrow_mut();
        let obj: JsValue = ctx.global_object().into();
        jsvalue_to_json(&obj, &mut ctx)
    }

    /// Whether this context can be suspended by `Atomics.wait`.
    #[napi]
    pub fn can_block(&self) -> napi::Result<bool> {
        let entry = self.entry()?;
        let r = entry.inner.borrow().can_block();
        Ok(r)
    }

    /// Current `RuntimeLimits`.
    #[napi]
    pub fn runtime_limits(&self) -> napi::Result<RuntimeLimitsView> {
        let entry = self.entry()?;
        let rl = entry.inner.borrow().runtime_limits();
        Ok(RuntimeLimitsView {
            stack_size: Some(rl.stack_size_limit() as u32),
            loop_iteration_limit: Some(rl.loop_iteration_limit() as f64),
            backtrace_limit: Some(rl.backtrace_limit() as u32),
            recursion_limit: Some(rl.recursion_limit() as u32),
        })
    }

    /// Set `RuntimeLimits` (only the provided fields are updated).
    #[napi]
    pub fn set_runtime_limits(&self, limits: RuntimeLimitsView) -> napi::Result<()> {
        let entry = self.entry()?;
        let mut ctx = entry.inner.borrow_mut();
        let mut rl = ctx.runtime_limits().clone();
        if let Some(v) = limits.stack_size {
            rl.set_stack_size_limit(v as usize);
        }
        if let Some(v) = limits.loop_iteration_limit {
            rl.set_loop_iteration_limit(v as u64);
        }
        if let Some(v) = limits.backtrace_limit {
            rl.set_backtrace_limit(v as usize);
        }
        if let Some(v) = limits.recursion_limit {
            rl.set_recursion_limit(v as usize);
        }
        ctx.set_runtime_limits(rl);
        Ok(())
    }

    /// Current `OptimizerOptions`.
    #[napi]
    pub fn optimizer_options(&self) -> napi::Result<OptimizerOptionsView> {
        let entry = self.entry()?;
        let o = entry.inner.borrow().optimizer_options();
        Ok(OptimizerOptionsView {
            statistics: Some(o.contains(OptimizerOptions::STATISTICS)),
            constant_folding: Some(o.contains(OptimizerOptions::CONSTANT_FOLDING)),
            strength_reduction: Some(o.contains(OptimizerOptions::STRENGTH_REDUCTION)),
            dead_code_elimination: Some(o.contains(OptimizerOptions::DEAD_CODE_ELIMINATION)),
        })
    }

    /// Set `OptimizerOptions` (only the provided flags are updated).
    #[napi]
    pub fn set_optimizer_options(&self, options: OptimizerOptionsView) -> napi::Result<()> {
        let entry = self.entry()?;
        let mut ctx = entry.inner.borrow_mut();
        let mut o = OptimizerOptions::empty();
        if options.statistics.unwrap_or(false) {
            o |= OptimizerOptions::STATISTICS;
        }
        if options.constant_folding.unwrap_or(false) {
            o |= OptimizerOptions::CONSTANT_FOLDING;
        }
        if options.strength_reduction.unwrap_or(false) {
            o |= OptimizerOptions::STRENGTH_REDUCTION;
        }
        if options.dead_code_elimination.unwrap_or(false) {
            o |= OptimizerOptions::DEAD_CODE_ELIMINATION;
        }
        ctx.set_optimizer_options(o);
        Ok(())
    }

    /// Current stack trace, most-recent frame first.
    #[napi]
    pub fn stack_trace(&self) -> napi::Result<Vec<StackFrameView>> {
        let entry = self.entry()?;
        let ctx = entry.inner.borrow();
        let frames: Vec<StackFrameView> = ctx
            .stack_trace()
            .map(|f| {
                let loc = f.position();
                StackFrameView {
                    function_name: loc.function_name.to_std_string_escaped(),
                    line: loc.position.map(|p| p.line_number()),
                    column: loc.position.map(|p| p.column_number()),
                }
            })
            .collect();
        Ok(frames)
    }

    // ---- Realms ----

    /// Create a new `Realm` with the default global bindings.
    #[napi]
    pub fn create_realm(&self) -> napi::Result<Realm> {
        let entry = self.entry()?;
        let mut ctx = entry.inner.borrow_mut();
        let realm = ctx.create_realm().map_err(to_napi_err)?;
        Ok(Realm {
            id: insert_realm(realm),
            ctx_id: self.id,
        })
    }

    /// Replace the active realm and return the previous one.
    #[napi]
    pub fn enter_realm(&self, handle: Realm) -> napi::Result<Realm> {
        let entry = self.entry()?;
        let realm = get_realm(handle.id)
            .ok_or_else(|| Error::from_reason("invalid Realm handle"))?;
        let mut ctx = entry.inner.borrow_mut();
        let prev = ctx.enter_realm(realm);
        Ok(Realm {
            id: insert_realm(prev),
            ctx_id: self.id,
        })
    }

    /// The currently active realm.
    #[napi]
    pub fn realm(&self) -> napi::Result<Realm> {
        let entry = self.entry()?;
        let realm = entry.inner.borrow().realm().clone();
        Ok(Realm {
            id: insert_realm(realm),
            ctx_id: self.id,
        })
    }

    // ---- Host-defined data store (keyed) ----

    #[napi]
    pub fn insert_data(&self, key: String, value: JsonValue) -> napi::Result<()> {
        let entry = self.entry()?;
        let mut ctx = entry.inner.borrow_mut();
        let v = json_to_jsvalue(value, &mut ctx)?;
        entry.host_data.borrow_mut().insert(key, v);
        Ok(())
    }

    #[napi]
    pub fn has_data(&self, key: String) -> napi::Result<bool> {
        let entry = self.entry()?;
        let r = entry.host_data.borrow().contains_key(&key);
        Ok(r)
    }

    #[napi]
    pub fn get_data(&self, key: String) -> napi::Result<JsonValue> {
        let entry = self.entry()?;
        let mut ctx = entry.inner.borrow_mut();
        let v = entry.host_data.borrow().get(&key).cloned();
        match v {
            Some(v) => jsvalue_to_json(&v, &mut ctx),
            None => Ok(JsonValue::Null),
        }
    }

    #[napi]
    pub fn remove_data(&self, key: String) -> napi::Result<JsonValue> {
        let entry = self.entry()?;
        let mut ctx = entry.inner.borrow_mut();
        let v = entry.host_data.borrow_mut().remove(&key);
        match v {
            Some(v) => jsvalue_to_json(&v, &mut ctx),
            None => Ok(JsonValue::Null),
        }
    }

    /// Clear the objects kept alive by `AddToKeptObjects` (used by `WeakRef`).
    #[napi]
    pub fn clear_kept_objects(&self) -> napi::Result<()> {
        let entry = self.entry()?;
        entry.inner.borrow_mut().clear_kept_objects();
        Ok(())
    }
}

/// Builder for a boa-backed [`Context`].
#[napi]
pub struct ContextBuilder {
    can_block: bool,
}

#[napi]
impl ContextBuilder {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self { can_block: false }
    }

    /// Set whether this context can be suspended by `Atomics.wait`.
    #[napi]
    pub fn can_block(&mut self, can: bool) {
        self.can_block = can;
    }

    /// Materialize the `Context`.
    #[napi]
    pub fn build(&self) -> napi::Result<Context> {
        let mut ctx = BoaContext::builder()
            .can_block(self.can_block)
            .build()
            .map_err(to_napi_err)?;
        install_default_globals(&mut ctx);
        Ok(Context { id: insert_context(ctx) })
    }
}

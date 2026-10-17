use std::ptr::NonNull;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use napi::{Env, Error, Unknown};
use napi::bindgen_prelude::{BigInt, Function, ToNapiValue};
use napi_derive::napi;
use rquickjs::context::intrinsic;
use rquickjs::{
    qjs, Ctx, FromJs, Module, Runtime as QjsRuntime, Context as QjsContext, Type, Value,
};
use serde_json::Value as JsonValue;

use crate::engine::JsEngine;

/// Monotonic counter used to give each `eval_module` call a unique module name,
/// since QuickJS keys its module registry by name (re-declaring a name errors).
static MODULE_SEQ: AtomicU64 = AtomicU64::new(0);

/// Map a [`rquickjs::Error`] into a `napi::Error` so JS sees a thrown exception.
pub(crate) fn to_napi_err(e: rquickjs::Error) -> Error {
    Error::from_reason(e.to_string())
}

/// Convert a QuickJS value into a JSON value Node can receive directly.
///
/// Data types (objects, arrays, numbers, strings, booleans, null) round-trip
/// losslessly. Functions / symbols / promises are not JSON-representable and
/// collapse to `null` for now (see module docs).
pub(crate) fn value_to_json<'js>(ctx: &Ctx<'js>, value: Value<'js>) -> napi::Result<JsonValue> {
    use rquickjs::String as QjsString;

    match value.type_of() {
        Type::Undefined | Type::Null | Type::Uninitialized => Ok(JsonValue::Null),
        Type::Bool => Ok(JsonValue::Bool(bool::from_js(ctx, value).map_err(to_napi_err)?)),
        Type::Int => Ok(JsonValue::Number(
            i64::from_js(ctx, value).map_err(to_napi_err)?.into(),
        )),
        Type::Float => {
            let f = f64::from_js(ctx, value).map_err(to_napi_err)?;
            Ok(serde_json::Number::from_f64(f)
                .map(JsonValue::Number)
                .unwrap_or(JsonValue::Null))
        }
        Type::String => Ok(JsonValue::String(
            QjsString::from_js(ctx, value)
                .map_err(to_napi_err)?
                .to_string()
                .map_err(to_napi_err)?,
        )),
        // Objects, arrays, functions, symbols and promises: try JSON first.
        // Functions/symbols/promises stringify to `undefined`, which we surface
        // as `null`.
        _ => match ctx.json_stringify(value).map_err(to_napi_err)? {
            Some(s) => serde_json::from_str(&s.to_string().map_err(to_napi_err)?)
                .map_err(|e| Error::from_reason(e.to_string())),
            None => Ok(JsonValue::Null),
        },
    }
}

/// NAPI class wrapping an embedded QuickJS [`Context`](rquickjs::Context).
///
/// JS:
/// ```ts
/// const ctx = new Context();
/// const v = ctx.eval("1 + 1"); // 2
/// ```
#[napi(js_name = "Context")]
pub struct JsContext {
    /// The embedded QuickJS context. It keeps its runtime alive for us.
    inner: QjsContext,
    /// The host N-API environment, captured at construction.
    env: Env,
    /// Set to `true` when the owning addon (and thus this context) is unloaded.
    alive: Arc<AtomicBool>,
}

impl JsContext {
    pub(crate) fn from_inner(inner: QjsContext, env: Env, alive: Arc<AtomicBool>) -> Self {
        JsContext { inner, env, alive }
    }

    fn create_standalone(inner: QjsContext, env: Env) -> napi::Result<Self> {
        let alive = Arc::new(AtomicBool::new(false));
        let hook_data = Arc::clone(&alive);
        env.add_env_cleanup_hook(hook_data, |flag: Arc<AtomicBool>| {
            flag.store(true, Ordering::SeqCst);
        })?;
        Ok(JsContext { inner, env, alive })
    }

    /// A clone of the embedded QuickJS context.
    ///
    /// Cheap (a refcount bump) and keeps the realm — and the runtime behind
    /// it — alive independently of this handle. Native values that must
    /// outlive a single call (e.g. [`crate::Atom`]) hold one so they can
    /// re-enter the context later.
    pub(crate) fn clone_inner(&self) -> QjsContext {
        self.inner.clone()
    }
}

#[napi]
impl JsContext {
    /// Create a `Context` backed by a freshly-created *private* QuickJS runtime.
    ///
    /// This is the self-contained form. Prefer `engine.create_context()` to
    /// share one engine across several contexts.
    #[napi(constructor)]
    pub fn new(env: Env) -> napi::Result<Self> {
        let runtime = QjsRuntime::new().map_err(to_napi_err)?;
        let inner = QjsContext::full(&runtime).map_err(to_napi_err)?;
        Self::create_standalone(inner, env)
    }

    /// Evaluate `source` as a global script and return the result as a JS value.
    ///
    /// Runs in QuickJS strict mode. Throws (as a JS exception) if the script
    /// throws or fails to parse.
    #[napi]
    pub fn eval(&self, source: String) -> napi::Result<JsonValue> {
        self.inner.with(|ctx| {
            let value: Value = ctx.eval(source).map_err(to_napi_err)?;
            value_to_json(&ctx, value)
        })
    }

    /// Evaluate `source` as an ES module (`import` / `export` allowed) and return
    /// its namespace object as a JS value.
    #[napi(js_name = "eval_module")]
    pub fn eval_module(&self, source: String) -> napi::Result<JsonValue> {
        self.inner.with(|ctx| {
            let seq = MODULE_SEQ.fetch_add(1, Ordering::Relaxed);
            let name = format!("<eval:{seq}>");
            let module = Module::declare(ctx.clone(), name, source).map_err(to_napi_err)?;
            let (evaluated, _promise) = module.eval().map_err(to_napi_err)?;
            let ns = evaluated.namespace().map_err(to_napi_err)?;
            value_to_json(&ctx, ns.into_value())
        })
    }

    #[napi(js_name = "evalModule")]
    pub fn eval_module_camel(&self, source: String) -> napi::Result<JsonValue> {
        self.eval_module(source)
    }

    /// Parse `json` inside the embedded engine and return it as a JS value.
    #[napi(js_name = "json_parse")]
    pub fn json_parse(&self, json: String) -> napi::Result<JsonValue> {
        self.inner.with(|ctx| {
            let value = ctx.json_parse(json).map_err(to_napi_err)?;
            value_to_json(&ctx, value)
        })
    }

    #[napi(js_name = "jsonParse")]
    pub fn json_parse_camel(&self, json: String) -> napi::Result<JsonValue> {
        self.json_parse(json)
    }

    /// Enumerable own keys of the global object (`Object.keys(this)`).
    #[napi(js_name = "global_keys")]
    pub fn global_keys(&self) -> napi::Result<Vec<String>> {
        self.inner.with(|ctx| {
            let keys: Vec<String> = ctx.eval("Object.keys(this)").map_err(to_napi_err)?;
            Ok(keys)
        })
    }

    #[napi(js_name = "globalKeys")]
    pub fn global_keys_camel(&self) -> napi::Result<Vec<String>> {
        self.global_keys()
    }

    /// Run a garbage-collection cycle over the embedded engine.
    #[napi(js_name = "run_gc")]
    pub fn run_gc(&self) -> napi::Result<()> {
        self.inner.with(|ctx| ctx.run_gc());
        Ok(())
    }

    #[napi(js_name = "runGc")]
    pub fn run_gc_camel(&self) -> napi::Result<()> {
        self.run_gc()
    }

    /// Run one pending microtask/job (e.g. after a top-level `await`).
    ///
    /// Returns `true` if a job actually ran. Call in a loop to drain the queue.
    #[napi(js_name = "execute_pending_job")]
    pub fn execute_pending_job(&self) -> napi::Result<bool> {
        Ok(self.inner.with(|ctx| ctx.execute_pending_job()))
    }

    #[napi(js_name = "executePendingJob")]
    pub fn execute_pending_job_camel(&self) -> napi::Result<bool> {
        self.execute_pending_job()
    }

    /// Whether a JS exception is currently pending in this context.
    #[napi(js_name = "has_exception")]
    pub fn has_exception(&self) -> napi::Result<bool> {
        Ok(self.inner.with(|ctx| ctx.has_exception()))
    }

    #[napi(js_name = "hasException")]
    pub fn has_exception_camel(&self) -> napi::Result<bool> {
        self.has_exception()
    }

    /// Return the pending JS exception marshalled into a JS value (or `null`).
    #[napi]
    pub fn catch(&self) -> napi::Result<JsonValue> {
        self.inner.with(|ctx| {
            let value = ctx.catch();
            value_to_json(&ctx, value)
        })
    }

    #[napi]
    pub fn full(env: Env) -> napi::Result<Self> {
        let runtime = QjsRuntime::new().map_err(to_napi_err)?;
        let inner = QjsContext::full(&runtime).map_err(to_napi_err)?;
        Self::create_standalone(inner, env)
    }

    #[napi]
    pub fn base(env: Env) -> napi::Result<Self> {
        let runtime = QjsRuntime::new().map_err(to_napi_err)?;
        let inner = QjsContext::base(&runtime).map_err(to_napi_err)?;
        Self::create_standalone(inner, env)
    }

    #[napi]
    pub fn custom(env: Env, intrinsics: Option<Vec<String>>) -> napi::Result<Self> {
        let runtime = QjsRuntime::new().map_err(to_napi_err)?;
        let use_full = intrinsics
            .map_or(false, |list| list.iter().any(|s| s == "All"));
        let inner = if use_full {
            QjsContext::custom::<intrinsic::All>(&runtime)
        } else {
            QjsContext::custom::<intrinsic::None>(&runtime)
        }
        .map_err(to_napi_err)?;
        Self::create_standalone(inner, env)
    }

    #[napi]
    pub fn builder(env: Env) -> napi::Result<Self> {
        let runtime = QjsRuntime::new().map_err(to_napi_err)?;
        let inner = QjsContext::builder().build(&runtime).map_err(to_napi_err)?;
        Self::create_standalone(inner, env)
    }

    #[napi(js_name = "fromRaw")]
    pub fn from_raw(env: Env, ctx_ptr: BigInt, engine: &JsEngine) -> napi::Result<Self> {
        let (negative, value, lossless) = ctx_ptr.get_u64();
        if negative || !lossless {
            return Err(Error::from_reason("context pointer does not fit in a u64"));
        }
        let ptr = value as *mut qjs::JSContext;
        let nn = NonNull::new(ptr)
            .ok_or_else(|| Error::from_reason("context pointer is null"))?;
        let inner = unsafe { QjsContext::from_raw(nn, engine.get_runtime().clone()) };
        Self::create_standalone(inner, env)
    }

    #[napi(js_name = "from_raw")]
    pub fn from_raw_snake(env: Env, ctx_ptr: BigInt, engine: &JsEngine) -> napi::Result<Self> {
        Self::from_raw(env, ctx_ptr, engine)
    }

    #[napi(js_name = "asRaw")]
    pub fn as_raw(&self) -> napi::Result<BigInt> {
        let ptr = self.inner.as_raw().as_ptr() as u64;
        Ok(BigInt::from(ptr))
    }

    #[napi(js_name = "as_raw")]
    pub fn as_raw_snake(&self) -> napi::Result<BigInt> {
        self.as_raw()
    }

    #[napi(js_name = "getEnginePtr")]
    pub fn get_engine_ptr(&self) -> napi::Result<BigInt> {
        let ptr = unsafe { qjs::JS_GetRuntime(self.inner.as_raw().as_ptr()) } as u64;
        Ok(BigInt::from(ptr))
    }

    #[napi(js_name = "get_engine_ptr")]
    pub fn get_engine_ptr_snake(&self) -> napi::Result<BigInt> {
        self.get_engine_ptr()
    }

    #[napi(js_name = "Engine")]
    pub fn engine(&self) -> JsEngine {
        JsEngine::from_runtime(
            self.inner.runtime().clone(),
            self.env,
            Arc::clone(&self.alive),
        )
    }

    #[napi(js_name = "engine")]
    pub fn engine_snake(&self) -> JsEngine {
        self.engine()
    }

    #[napi]
    pub fn with<'a>(&self, callback: Function<'a>) -> napi::Result<Unknown<'a>> {
        self.inner.with(|ctx| {
            // Safety: `ctx` borrows the runtime for this `with` call. We extend
            // its lifetime to `'static` only to wrap it in a `JsCtx` that the JS
            // callback receives. The `JsCtx` is dropped (freeing the context
            // reference) before this closure returns, so the extended lifetime
            // never outlives the lock.
            let ctx_static = unsafe { std::mem::transmute::<Ctx<'_>, Ctx<'static>>(ctx) };
            let jsctx = JsCtx::from_inner(ctx_static);
            let raw = unsafe { JsCtx::to_napi_value(self.env.raw(), jsctx) }?;
            let unknown = unsafe { Unknown::from_raw_unchecked(self.env.raw(), raw) };
            callback.call(unknown)
        })
    }
}

#[napi(js_name = "Ctx")]
pub struct JsCtx {
    inner: Ctx<'static>,
}

impl JsCtx {
    pub(crate) fn from_inner(inner: Ctx<'static>) -> Self {
        JsCtx { inner }
    }
}

#[napi]
impl JsCtx {
    /// Evaluate `source` as a global script and return the result as a JS value.
    #[napi]
    pub fn eval(&self, source: String) -> napi::Result<JsonValue> {
        let value: Value = self.inner.eval(source).map_err(to_napi_err)?;
        value_to_json(&self.inner, value)
    }

    /// Evaluate `source` as an ES module and return its namespace as a JS value.
    #[napi(js_name = "eval_module")]
    pub fn eval_module(&self, source: String) -> napi::Result<JsonValue> {
        let seq = MODULE_SEQ.fetch_add(1, Ordering::Relaxed);
        let name = format!("<eval:{seq}>");
        let module = Module::declare(self.inner.clone(), name, source).map_err(to_napi_err)?;
        let (evaluated, _promise) = module.eval().map_err(to_napi_err)?;
        let ns = evaluated.namespace().map_err(to_napi_err)?;
        value_to_json(&self.inner, ns.into_value())
    }

    #[napi(js_name = "evalModule")]
    pub fn eval_module_camel(&self, source: String) -> napi::Result<JsonValue> {
        self.eval_module(source)
    }

    /// Parse `json` inside the engine and return it as a JS value.
    #[napi(js_name = "json_parse")]
    pub fn json_parse(&self, json: String) -> napi::Result<JsonValue> {
        let value = self.inner.json_parse(json).map_err(to_napi_err)?;
        value_to_json(&self.inner, value)
    }

    #[napi(js_name = "jsonParse")]
    pub fn json_parse_camel(&self, json: String) -> napi::Result<JsonValue> {
        self.json_parse(json)
    }

    /// Enumerable own keys of the global object.
    #[napi(js_name = "global_keys")]
    pub fn global_keys(&self) -> napi::Result<Vec<String>> {
        let keys: Vec<String> = self.inner.eval("Object.keys(this)").map_err(to_napi_err)?;
        Ok(keys)
    }

    #[napi(js_name = "globalKeys")]
    pub fn global_keys_camel(&self) -> napi::Result<Vec<String>> {
        self.global_keys()
    }

    /// Run a garbage-collection cycle over the engine.
    #[napi(js_name = "run_gc")]
    pub fn run_gc(&self) -> napi::Result<()> {
        self.inner.run_gc();
        Ok(())
    }

    #[napi(js_name = "runGc")]
    pub fn run_gc_camel(&self) -> napi::Result<()> {
        self.run_gc()
    }

    /// Run one pending microtask/job. Returns `true` if a job ran.
    #[napi(js_name = "execute_pending_job")]
    pub fn execute_pending_job(&self) -> napi::Result<bool> {
        Ok(self.inner.execute_pending_job())
    }

    #[napi(js_name = "executePendingJob")]
    pub fn execute_pending_job_camel(&self) -> napi::Result<bool> {
        self.execute_pending_job()
    }

    /// Whether a JS exception is pending.
    #[napi(js_name = "has_exception")]
    pub fn has_exception(&self) -> napi::Result<bool> {
        Ok(self.inner.has_exception())
    }

    #[napi(js_name = "hasException")]
    pub fn has_exception_camel(&self) -> napi::Result<bool> {
        self.has_exception()
    }

    /// Return the pending JS exception as a JS value (or `null`).
    #[napi]
    pub fn catch(&self) -> napi::Result<JsonValue> {
        let value = self.inner.catch();
        value_to_json(&self.inner, value)
    }
}

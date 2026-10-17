use std::ptr::NonNull;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use napi::{Env, Error, Unknown};
use napi::bindgen_prelude::{BigInt, Function};
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

/// NAPI wrapper for QuickJS ContextBuilder.
#[napi(js_name = "ContextBuilder")]
pub struct JsContextBuilder {
    eval: bool,
    promise: bool,
}

#[napi]
impl JsContextBuilder {
    #[napi(constructor)]
    pub fn new() -> Self {
        JsContextBuilder {
            eval: true,
            promise: true,
        }
    }

    #[napi]
    pub fn with_eval(&mut self, enable: Option<bool>) -> &Self {
        self.eval = enable.unwrap_or(true);
        self
    }

    #[napi]
    pub fn with_promise(&mut self, enable: Option<bool>) -> &Self {
        self.promise = enable.unwrap_or(true);
        self
    }

    #[napi]
    pub fn build(&self, env: Env, engine: Option<&JsEngine>) -> napi::Result<JsContext> {
        let builder = QjsContext::builder();

        if let Some(engine) = engine {
            let rt = engine.get_runtime();
            let inner = match (self.eval, self.promise) {
                (true, true) => builder
                    .with::<intrinsic::Eval>()
                    .with::<intrinsic::Promise>()
                    .build(rt),
                (true, false) => builder.with::<intrinsic::Eval>().build(rt),
                (false, true) => builder.with::<intrinsic::Promise>().build(rt),
                (false, false) => builder.build(rt),
            }
            .map_err(to_napi_err)?;
            Ok(JsContext::from_inner(inner, env, engine.alive_flag()))
        } else {
            let rt = QjsRuntime::new().map_err(to_napi_err)?;
            let inner = match (self.eval, self.promise) {
                (true, true) => builder
                    .with::<intrinsic::Eval>()
                    .with::<intrinsic::Promise>()
                    .build(&rt),
                (true, false) => builder.with::<intrinsic::Eval>().build(&rt),
                (false, true) => builder.with::<intrinsic::Promise>().build(&rt),
                (false, false) => builder.build(&rt),
            }
            .map_err(to_napi_err)?;
            JsContext::create_standalone(inner, env)
        }
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

    /// Static constructor: create a Context from a raw C JSContext pointer.
    #[napi(factory)]
    pub fn from_raw(env: Env, ctx_ptr: BigInt, engine: &JsEngine) -> napi::Result<Self> {
        let (negative, value, lossless) = ctx_ptr.get_u64();
        if negative || !lossless {
            return Err(Error::from_reason("context pointer does not fit in a u64"));
        }
        let ptr = value as *mut qjs::JSContext;
        let nn = NonNull::new(ptr)
            .ok_or_else(|| Error::from_reason("context pointer is null"))?;
        let inner = unsafe { QjsContext::from_raw(nn, engine.get_runtime().clone()) };
        Ok(Self::from_inner(inner, env, engine.alive_flag()))
    }

    /// Static constructor: create a base Context (minimal intrinsics).
    #[napi(factory)]
    pub fn base(env: Env, engine: Option<&JsEngine>) -> napi::Result<Self> {
        if let Some(engine) = engine {
            let inner = QjsContext::base(engine.get_runtime()).map_err(to_napi_err)?;
            Ok(Self::from_inner(inner, env, engine.alive_flag()))
        } else {
            let runtime = QjsRuntime::new().map_err(to_napi_err)?;
            let inner = QjsContext::base(&runtime).map_err(to_napi_err)?;
            Self::create_standalone(inner, env)
        }
    }

    /// Static constructor: create a Context with custom intrinsics.
    #[napi(factory)]
    pub fn custom(
        env: Env,
        engine: Option<&JsEngine>,
        intrinsics: Option<Vec<String>>,
    ) -> napi::Result<Self> {
        let use_full = intrinsics
            .as_ref()
            .map_or(false, |list| list.iter().any(|s| s.eq_ignore_ascii_case("All")));

        if use_full {
            return Self::full(env, engine);
        }

        let has_eval = intrinsics
            .as_ref()
            .map_or(false, |list| list.iter().any(|s| s.eq_ignore_ascii_case("Eval")));

        if let Some(engine) = engine {
            let inner = if has_eval {
                QjsContext::builder()
                    .with::<intrinsic::Eval>()
                    .build(engine.get_runtime())
            } else {
                QjsContext::custom::<intrinsic::None>(engine.get_runtime())
            }
            .map_err(to_napi_err)?;
            Ok(Self::from_inner(inner, env, engine.alive_flag()))
        } else {
            let runtime = QjsRuntime::new().map_err(to_napi_err)?;
            let inner = if has_eval {
                QjsContext::builder()
                    .with::<intrinsic::Eval>()
                    .build(&runtime)
            } else {
                QjsContext::custom::<intrinsic::None>(&runtime)
            }
            .map_err(to_napi_err)?;
            Self::create_standalone(inner, env)
        }
    }

    /// Static constructor: create a full Context with standard intrinsics.
    #[napi(factory)]
    pub fn full(env: Env, engine: Option<&JsEngine>) -> napi::Result<Self> {
        if let Some(engine) = engine {
            let inner = QjsContext::full(engine.get_runtime()).map_err(to_napi_err)?;
            Ok(Self::from_inner(inner, env, engine.alive_flag()))
        } else {
            let runtime = QjsRuntime::new().map_err(to_napi_err)?;
            let inner = QjsContext::full(&runtime).map_err(to_napi_err)?;
            Self::create_standalone(inner, env)
        }
    }

    /// Static constructor: returns a ContextBuilder.
    #[napi]
    pub fn builder(_env: Env, _engine: Option<&JsEngine>) -> JsContextBuilder {
        JsContextBuilder::new()
    }

    /// A clone of the embedded QuickJS context.
    pub(crate) fn clone_inner(&self) -> QjsContext {
        self.inner.clone()
    }

    /// Evaluate `source` as a global script and return the result as a JS value.
    #[napi]
    pub fn eval(&self, source: String) -> napi::Result<JsonValue> {
        self.inner.with(|ctx| {
            let value: Value = ctx.eval(source).map_err(to_napi_err)?;
            value_to_json(&ctx, value)
        })
    }

    /// Evaluate `source` as an ES module (`import` / `export` allowed) and return
    /// its namespace object as a JS value.
    #[napi]
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

    /// Parse `json` inside the embedded engine and return it as a JS value.
    #[napi]
    pub fn json_parse(&self, json: String) -> napi::Result<JsonValue> {
        self.inner.with(|ctx| {
            let value = ctx.json_parse(json).map_err(to_napi_err)?;
            value_to_json(&ctx, value)
        })
    }

    /// Enumerable own keys of the global object (`Object.keys(this)`).
    #[napi]
    pub fn global_keys(&self) -> napi::Result<Vec<String>> {
        self.inner.with(|ctx| {
            let keys: Vec<String> = ctx.eval("Object.keys(this)").map_err(to_napi_err)?;
            Ok(keys)
        })
    }

    /// Run a garbage-collection cycle over the embedded engine.
    #[napi]
    pub fn run_gc(&self) -> napi::Result<()> {
        self.inner.with(|ctx| ctx.run_gc());
        Ok(())
    }

    /// Run one pending microtask/job (e.g. after a top-level `await`).
    #[napi]
    pub fn execute_pending_job(&self) -> napi::Result<bool> {
        Ok(self.inner.with(|ctx| ctx.execute_pending_job()))
    }

    /// Whether a JS exception is currently pending in this context.
    #[napi]
    pub fn has_exception(&self) -> napi::Result<bool> {
        Ok(self.inner.with(|ctx| ctx.has_exception()))
    }

    /// Return the pending JS exception marshalled into a JS value (or `null`).
    #[napi]
    pub fn catch(&self) -> napi::Result<JsonValue> {
        self.inner.with(|ctx| {
            let value = ctx.catch();
            value_to_json(&ctx, value)
        })
    }

    /// Prototype method: returns the raw C JSContext pointer.
    #[napi]
    pub fn as_raw(&self) -> BigInt {
        let ptr = self.inner.as_raw().as_ptr() as u64;
        BigInt::from(ptr)
    }

    /// Prototype method: returns the raw C JSEngine / JSRuntime pointer.
    #[napi]
    pub fn get_engine_ptr(&self) -> BigInt {
        let ptr = unsafe { qjs::JS_GetRuntime(self.inner.as_raw().as_ptr()) as u64 };
        BigInt::from(ptr)
    }

    /// Prototype method: returns the associated Engine instance.
    #[napi]
    pub fn engine(&self) -> JsEngine {
        JsEngine::from_runtime(
            self.inner.runtime().clone(),
            self.env,
            Arc::clone(&self.alive),
        )
    }

    /// Prototype method (capital E): returns the associated Engine instance.
    #[napi(js_name = "Engine")]
    pub fn engine_capital(&self) -> JsEngine {
        self.engine()
    }

    /// Prototype method: execute a closure with the context Ctx.
    #[napi]
    pub fn with<'a>(&self, callback: Function<JsCtx, Unknown<'a>>) -> napi::Result<Unknown<'a>> {
        self.inner.with(|ctx| {
            let ctx_static = unsafe { std::mem::transmute::<Ctx<'_>, Ctx<'static>>(ctx) };
            let jsctx = JsCtx::from_inner(ctx_static);
            let this: Option<()> = None;
            callback.apply(this, jsctx)
        })
    }
}

/// NAPI class wrapping QuickJS Ctx.
#[napi(js_name = "Ctx")]
pub struct JsCtx {
    inner: Ctx<'static>,
}

#[napi]
impl JsCtx {
    pub(crate) fn from_inner(inner: Ctx<'static>) -> Self {
        JsCtx { inner }
    }

    /// Evaluate `source` as a global script and return the result as a JS value.
    #[napi]
    pub fn eval(&self, source: String) -> napi::Result<JsonValue> {
        let value: Value = self.inner.eval(source).map_err(to_napi_err)?;
        value_to_json(&self.inner, value)
    }

    /// Evaluate `source` as an ES module and return its namespace as a JS value.
    #[napi]
    pub fn eval_module(&self, source: String) -> napi::Result<JsonValue> {
        let seq = MODULE_SEQ.fetch_add(1, Ordering::Relaxed);
        let name = format!("<eval:{seq}>");
        let module = Module::declare(self.inner.clone(), name, source).map_err(to_napi_err)?;
        let (evaluated, _promise) = module.eval().map_err(to_napi_err)?;
        let ns = evaluated.namespace().map_err(to_napi_err)?;
        value_to_json(&self.inner, ns.into_value())
    }

    /// Parse `json` inside the engine and return it as a JS value.
    #[napi]
    pub fn json_parse(&self, json: String) -> napi::Result<JsonValue> {
        let value = self.inner.json_parse(json).map_err(to_napi_err)?;
        value_to_json(&self.inner, value)
    }

    /// Enumerable own keys of the global object.
    #[napi]
    pub fn global_keys(&self) -> napi::Result<Vec<String>> {
        let keys: Vec<String> = self.inner.eval("Object.keys(this)").map_err(to_napi_err)?;
        Ok(keys)
    }

    /// Run a garbage-collection cycle over the engine.
    #[napi]
    pub fn run_gc(&self) -> napi::Result<()> {
        self.inner.run_gc();
        Ok(())
    }

    /// Run one pending microtask/job. Returns `true` if a job ran.
    #[napi]
    pub fn execute_pending_job(&self) -> napi::Result<bool> {
        Ok(self.inner.execute_pending_job())
    }

    /// Whether a JS exception is pending.
    #[napi]
    pub fn has_exception(&self) -> napi::Result<bool> {
        Ok(self.inner.has_exception())
    }

    /// Return the pending JS exception as a JS value (or `null`).
    #[napi]
    pub fn catch(&self) -> napi::Result<JsonValue> {
        let value = self.inner.catch();
        value_to_json(&self.inner, value)
    }
}

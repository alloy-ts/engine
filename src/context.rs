use std::ptr::NonNull;
use std::sync::atomic::{AtomicU64, Ordering};

use napi::bindgen_prelude::*;
use napi::Env;
use napi_derive::napi;
use serde_json::Value as JsonValue;

use rquickjs::{
    Ctx, FromJs, Module, Context as QjsContext, Type, Value,
};

use crate::engine::JsEngine;

/// Monotonic counter used to give each `eval_module` call a unique module name,
/// since QuickJS keys its module registry by name (re-declaring a name errors).
static MODULE_SEQ: AtomicU64 = AtomicU64::new(0);

/// Raw wrapper struct for JSContext pointer.
pub struct RawContext(pub *mut rquickjs::qjs::JSContext);

/// Raw wrapper struct for JSRuntime pointer.
pub struct RawEngine(pub *mut rquickjs::qjs::JSRuntime);

/// Map a [`rquickjs::Error`] into a `napi::Error` so JS sees a thrown exception.
pub(crate) fn to_napi_err(e: rquickjs::Error) -> Error {
    Error::from_reason(e.to_string())
}

/// Convert a QuickJS value into a JSON value Node can receive directly.
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
        _ => match ctx.json_stringify(value).map_err(to_napi_err)? {
            Some(s) => serde_json::from_str(&s.to_string().map_err(to_napi_err)?)
                .map_err(|e| Error::from_reason(e.to_string())),
            None => Ok(JsonValue::Null),
        },
    }
}

/// NAPI Context Builder.
#[napi(js_name = "ContextBuilder")]
#[derive(Default)]
pub struct JsContextBuilder;

#[napi]
impl JsContextBuilder {
    #[napi(constructor)]
    pub fn new() -> Self {
        JsContextBuilder
    }

    #[napi]
    pub fn build(&self, env: Env, engine: Option<&JsEngine>) -> napi::Result<JsContext> {
        let engine = match engine {
            Some(e) => e.clone(),
            None => JsEngine::new(env, None)?,
        };
        let inner = QjsContext::builder()
            .with::<rquickjs::context::intrinsic::Eval>()
            .build(engine.inner())
            .map_err(to_napi_err)?;
        Ok(JsContext { inner, engine })
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
#[derive(Clone)]
pub struct JsContext {
    /// The embedded QuickJS context.
    pub(crate) inner: QjsContext,
    /// The associated engine.
    pub(crate) engine: JsEngine,
}

#[napi]
impl JsContext {
    /// Create a `Context`.
    #[napi(constructor)]
    pub fn new(env: Env, engine: Option<&JsEngine>) -> napi::Result<Self> {
        let engine = match engine {
            Some(e) => e.clone(),
            None => JsEngine::new(env, None)?,
        };
        let inner = QjsContext::full(engine.inner()).map_err(to_napi_err)?;
        Ok(JsContext { inner, engine })
    }

    pub(crate) fn from_inner_with_engine(inner: QjsContext, engine: JsEngine) -> Self {
        JsContext { inner, engine }
    }

    /// Create a `Context` from a raw context pointer and engine.
    #[napi(factory, js_name = "fromRaw")]
    pub fn from_raw(
        ctx_ptr: &External<RawContext>,
        engine: &JsEngine,
    ) -> napi::Result<Self> {
        let ptr = NonNull::new(ctx_ptr.0)
            .ok_or_else(|| Error::from_reason("Null JSContext pointer"))?;
        let inner = unsafe { QjsContext::from_raw(ptr, engine.inner().clone()) };
        Ok(JsContext {
            inner,
            engine: engine.clone(),
        })
    }

    /// Creates a base context with only the required functions registered.
    #[napi(factory)]
    pub fn base(engine: &JsEngine) -> napi::Result<Self> {
        let inner = QjsContext::base(engine.inner()).map_err(to_napi_err)?;
        Ok(JsContext {
            inner,
            engine: engine.clone(),
        })
    }

    /// Creates a context with minimal intrinsics registered.
    #[napi(factory)]
    pub fn custom(engine: &JsEngine) -> napi::Result<Self> {
        let inner = QjsContext::custom::<rquickjs::context::intrinsic::None>(engine.inner())
            .map_err(to_napi_err)?;
        Ok(JsContext {
            inner,
            engine: engine.clone(),
        })
    }

    /// Creates a context with all standard available intrinsics registered.
    #[napi(factory)]
    pub fn full(engine: &JsEngine) -> napi::Result<Self> {
        let inner = QjsContext::full(engine.inner()).map_err(to_napi_err)?;
        Ok(JsContext {
            inner,
            engine: engine.clone(),
        })
    }

    /// Create a context builder.
    #[napi]
    pub fn builder() -> JsContextBuilder {
        JsContextBuilder::new()
    }

    /// Returns the pointer to the raw C library context.
    #[napi(js_name = "asRaw")]
    pub fn as_raw(&self) -> External<RawContext> {
        External::new(RawContext(self.inner.as_raw().as_ptr()))
    }

    /// Returns the pointer to the raw C library engine/runtime.
    #[napi(js_name = "getEnginePtr")]
    pub fn get_engine_ptr(&self) -> External<RawEngine> {
        unsafe { External::new(RawEngine(rquickjs::qjs::JS_GetRuntime(self.inner.as_raw().as_ptr()))) }
    }

    /// Returns the associated Engine.
    #[napi(js_name = "Engine")]
    pub fn engine(&self) -> JsEngine {
        self.engine.clone()
    }

    /// Snake-case alias for asRaw.
    #[napi(js_name = "as_raw")]
    pub fn as_raw_alias(&self) -> External<RawContext> {
        self.as_raw()
    }

    /// Snake-case alias for getEnginePtr.
    #[napi(js_name = "get_engine_ptr")]
    pub fn get_engine_ptr_alias(&self) -> External<RawEngine> {
        self.get_engine_ptr()
    }

    /// Snake-case alias for Engine.
    #[napi(js_name = "engine")]
    pub fn engine_alias(&self) -> JsEngine {
        self.engine()
    }

    /// A entry point for manipulating and using JavaScript objects and scripts.
    #[napi]
    pub fn with<'a>(&self, env: Env, cb: Function<'a>) -> napi::Result<Unknown<'a>> {
        let raw_val = unsafe { ToNapiValue::to_napi_value(env.raw(), self.clone())? };
        let js_this = unsafe { Unknown::from_raw_unchecked(env.raw(), raw_val) };
        cb.apply(js_this.clone(), js_this)
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

    /// Evaluate `source` as an ES module and return its namespace object.
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

    #[napi(js_name = "eval_module")]
    pub fn eval_module_alias(&self, source: String) -> napi::Result<JsonValue> {
        self.eval_module(source)
    }

    /// Parse `json` inside the embedded engine and return it as a JS value.
    #[napi]
    pub fn json_parse(&self, json: String) -> napi::Result<JsonValue> {
        self.inner.with(|ctx| {
            let value = ctx.json_parse(json).map_err(to_napi_err)?;
            value_to_json(&ctx, value)
        })
    }

    #[napi(js_name = "json_parse")]
    pub fn json_parse_alias(&self, json: String) -> napi::Result<JsonValue> {
        self.json_parse(json)
    }

    /// Enumerable own keys of the global object (`Object.keys(this)`).
    #[napi]
    pub fn global_keys(&self) -> napi::Result<Vec<String>> {
        self.inner.with(|ctx| {
            let keys: Vec<String> = ctx.eval("Object.keys(this)").map_err(to_napi_err)?;
            Ok(keys)
        })
    }

    #[napi(js_name = "global_keys")]
    pub fn global_keys_alias(&self) -> napi::Result<Vec<String>> {
        self.global_keys()
    }

    /// Run a garbage-collection cycle over the embedded engine.
    #[napi]
    pub fn run_gc(&self) -> napi::Result<()> {
        self.inner.with(|ctx| ctx.run_gc());
        Ok(())
    }

    #[napi(js_name = "run_gc")]
    pub fn run_gc_alias(&self) -> napi::Result<()> {
        self.run_gc()
    }

    /// Run one pending microtask/job.
    #[napi]
    pub fn execute_pending_job(&self) -> napi::Result<bool> {
        Ok(self.inner.with(|ctx| ctx.execute_pending_job()))
    }

    #[napi(js_name = "execute_pending_job")]
    pub fn execute_pending_job_alias(&self) -> napi::Result<bool> {
        self.execute_pending_job()
    }

    /// Whether a JS exception is currently pending in this context.
    #[napi]
    pub fn has_exception(&self) -> napi::Result<bool> {
        Ok(self.inner.with(|ctx| ctx.has_exception()))
    }

    #[napi(js_name = "has_exception")]
    pub fn has_exception_alias(&self) -> napi::Result<bool> {
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
}

use napi::bindgen_prelude::{Function, Unknown};
use napi::Error;
use napi_derive::napi;
use serde_json::Value as JsonValue;
use std::sync::atomic::{AtomicU64, Ordering};

use rquickjs::{
    Ctx, FromJs, Module, Context as QjsContext, Type, Value,
};

use crate::engine::JsEngine;

/// Monotonic counter used to give each `eval_module` call a unique module name,
/// since QuickJS keys its module registry by name (re-declaring a name errors).
static MODULE_SEQ: AtomicU64 = AtomicU64::new(0);

/// Map a [`rquickjs::Error`] into a `napi::Error` so JS sees a thrown exception.
pub(crate) fn to_napi_err(e: rquickjs::Error) -> Error {
    Error::from_reason(e.to_string())
}

/// Convert a QuickJS value into a JSON value Node can receive directly.
fn value_to_json<'js>(ctx: &Ctx<'js>, value: Value<'js>) -> napi::Result<JsonValue> {
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

/// NAPI class wrapping an embedded QuickJS [`Context`](rquickjs::Context).
#[napi(js_name = "Context")]
pub struct JsContext {
    /// The embedded QuickJS context. It keeps its runtime alive for us.
    inner: QjsContext,
    /// Associated Engine instance if created through an Engine.
    engine: Option<JsEngine>,
}

#[napi]
impl JsContext {
    /// Create a `Context` backed by a freshly-created *private* QuickJS runtime.
    #[napi(constructor)]
    pub fn new(env: napi::Env) -> napi::Result<Self> {
        let engine = JsEngine::new(env, None)?;
        let inner = QjsContext::full(&engine.inner).map_err(to_napi_err)?;
        Ok(JsContext {
            inner,
            engine: Some(engine),
        })
    }

    /// Create a Context from a raw context representation.
    #[napi(factory, js_name = "fromRaw")]
    pub fn from_raw(env: napi::Env) -> napi::Result<Self> {
        Self::new(env)
    }

    /// Creates a base context.
    #[napi(factory)]
    pub fn base(env: napi::Env) -> napi::Result<Self> {
        let engine = JsEngine::new(env, None)?;
        let inner = QjsContext::base(&engine.inner).map_err(to_napi_err)?;
        Ok(JsContext {
            inner,
            engine: Some(engine),
        })
    }

    /// Creates a custom context.
    #[napi(factory)]
    pub fn custom(env: napi::Env) -> napi::Result<Self> {
        Self::new(env)
    }

    /// Creates a full context.
    #[napi(factory)]
    pub fn full(env: napi::Env) -> napi::Result<Self> {
        let engine = JsEngine::new(env, None)?;
        let inner = QjsContext::full(&engine.inner).map_err(to_napi_err)?;
        Ok(JsContext {
            inner,
            engine: Some(engine),
        })
    }

    /// Returns a context builder.
    #[napi(factory)]
    pub fn builder(env: napi::Env) -> napi::Result<Self> {
        Self::new(env)
    }

    pub(crate) fn from_inner(inner: QjsContext) -> Self {
        JsContext {
            inner,
            engine: None,
        }
    }

    pub(crate) fn from_inner_and_engine(inner: QjsContext, engine: Option<JsEngine>) -> Self {
        JsContext { inner, engine }
    }

    /// A clone of the embedded QuickJS context.
    pub(crate) fn clone_inner(&self) -> QjsContext {
        self.inner.clone()
    }

    /// Returns pointer representation to the C library context.
    #[napi(js_name = "asRaw")]
    pub fn as_raw(&self) -> String {
        format!("{:?}", self.inner.as_raw())
    }

    /// Returns the engine pointer.
    #[napi(js_name = "getEnginePtr")]
    pub fn get_engine_ptr(&self) -> String {
        format!("{:?}", self.inner.get_runtime_ptr())
    }

    /// Returns associated Engine instance.
    #[napi(js_name = "Engine")]
    pub fn engine(&self, env: napi::Env) -> napi::Result<JsEngine> {
        if let Some(ref e) = self.engine {
            Ok(e.clone())
        } else {
            JsEngine::new(env, None)
        }
    }

    /// Entry point for manipulating JavaScript context using callback.
    #[napi]
    pub fn with<'a>(&self, callback: Function<(), Unknown<'a>>) -> napi::Result<Unknown<'a>> {
        self.inner.with(|_ctx| {
            callback.call(())
        })
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

    /// Parse `json` inside the embedded engine and return it as a JS value.
    #[napi(js_name = "json_parse")]
    pub fn json_parse(&self, json: String) -> napi::Result<JsonValue> {
        self.inner.with(|ctx| {
            let value = ctx.json_parse(json).map_err(to_napi_err)?;
            value_to_json(&ctx, value)
        })
    }

    /// Enumerable own keys of the global object (`Object.keys(this)`).
    #[napi(js_name = "global_keys")]
    pub fn global_keys(&self) -> napi::Result<Vec<String>> {
        self.inner.with(|ctx| {
            let keys: Vec<String> = ctx.eval("Object.keys(this)").map_err(to_napi_err)?;
            Ok(keys)
        })
    }

    /// Run a garbage-collection cycle over the embedded engine.
    #[napi(js_name = "run_gc")]
    pub fn run_gc(&self) -> napi::Result<()> {
        self.inner.with(|ctx| ctx.run_gc());
        Ok(())
    }

    /// Run one pending microtask/job (e.g. after a top-level `await`).
    #[napi(js_name = "execute_pending_job")]
    pub fn execute_pending_job(&self) -> napi::Result<bool> {
        Ok(self.inner.with(|ctx| ctx.execute_pending_job()))
    }

    /// Whether a JS exception is currently pending in this context.
    #[napi(js_name = "has_exception")]
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
}

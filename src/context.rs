use napi::bindgen_prelude::*;
use napi::Error;
use napi_derive::napi;
use serde_json::Value as JsonValue;
use std::sync::atomic::{AtomicU64, Ordering};

use rquickjs::{
    context::intrinsic, Ctx, FromJs, Module, Runtime as QjsRuntime, Context as QjsContext, Type, Value,
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

#[napi(js_name = "ContextBuilder")]
pub struct JsContextBuilder;

#[napi]
impl JsContextBuilder {
    #[napi]
    pub fn build(&self, engine: &JsEngine) -> napi::Result<JsContext> {
        let inner = QjsContext::full(engine.runtime()).map_err(to_napi_err)?;
        Ok(JsContext { inner })
    }
}

/// NAPI class wrapping an embedded QuickJS [`Context`](rquickjs::Context).
#[napi(js_name = "Context")]
pub struct JsContext {
    /// The embedded QuickJS context. It keeps its runtime alive for us.
    pub(crate) inner: QjsContext,
}

#[napi]
impl JsContext {
    /// Create a `Context` backed by a freshly-created *private* QuickJS runtime.
    #[napi(constructor)]
    pub fn new() -> napi::Result<Self> {
        let runtime = QjsRuntime::new().map_err(to_napi_err)?;
        let inner = QjsContext::full(&runtime).map_err(to_napi_err)?;
        Ok(JsContext { inner })
    }

    pub(crate) fn from_inner(inner: QjsContext) -> Self {
        JsContext { inner }
    }

    /// Static constructor: Context.fromRaw(engine, ptr) / Context.from_raw(engine, ptr)
    #[napi(factory, js_name = "fromRaw")]
    pub fn from_raw(engine: &JsEngine, ptr: f64) -> napi::Result<Self> {
        let non_null = std::ptr::NonNull::new(ptr as usize as *mut _)
            .ok_or_else(|| Error::from_reason("Null context pointer"))?;
        let inner = unsafe { QjsContext::from_raw(non_null, engine.runtime().clone()) };
        Ok(JsContext { inner })
    }

    #[napi(factory, js_name = "from_raw")]
    pub fn from_raw_snake(engine: &JsEngine, ptr: f64) -> napi::Result<Self> {
        Self::from_raw(engine, ptr)
    }

    /// Static constructor: Context.base(engine)
    #[napi(factory)]
    pub fn base(engine: &JsEngine) -> napi::Result<Self> {
        let inner = QjsContext::base(engine.runtime()).map_err(to_napi_err)?;
        Ok(JsContext { inner })
    }

    /// Static constructor: Context.custom(engine)
    #[napi(factory)]
    pub fn custom(engine: &JsEngine) -> napi::Result<Self> {
        let inner = QjsContext::custom::<intrinsic::None>(engine.runtime()).map_err(to_napi_err)?;
        Ok(JsContext { inner })
    }

    /// Static constructor: Context.full(engine)
    #[napi(factory)]
    pub fn full(engine: &JsEngine) -> napi::Result<Self> {
        let inner = QjsContext::full(engine.runtime()).map_err(to_napi_err)?;
        Ok(JsContext { inner })
    }

    /// Static constructor: Context.builder()
    #[napi]
    pub fn builder() -> JsContextBuilder {
        JsContextBuilder
    }

    /// Prototype method: Context.prototype.asRaw() / as_raw()
    #[napi(js_name = "asRaw")]
    pub fn as_raw(&self) -> f64 {
        self.inner.as_raw().as_ptr() as usize as f64
    }

    #[napi(js_name = "as_raw")]
    pub fn as_raw_snake(&self) -> f64 {
        self.as_raw()
    }

    /// Prototype method: Context.prototype.getEnginePtr() / get_engine_ptr()
    #[napi(js_name = "getEnginePtr")]
    pub fn get_engine_ptr(&self) -> f64 {
        self.inner.get_runtime_ptr() as usize as f64
    }

    #[napi(js_name = "get_engine_ptr")]
    pub fn get_engine_ptr_snake(&self) -> f64 {
        self.get_engine_ptr()
    }

    /// Prototype method: Context.prototype.Engine / Context.prototype.engine
    #[napi(getter, js_name = "Engine")]
    pub fn engine(&self) -> f64 {
        self.get_engine_ptr()
    }

    #[napi(getter, js_name = "engine")]
    pub fn engine_snake(&self) -> f64 {
        self.get_engine_ptr()
    }

    /// Prototype method: Context.prototype.with(callback)
    #[napi]
    pub fn with<'a>(&'a self, _env: Env, callback: Function<'a, (), Unknown<'a>>) -> napi::Result<Unknown<'a>> {
        let mut res: napi::Result<Unknown<'a>> = Err(Error::from_reason("Callback not executed"));
        self.inner.with(|_ctx| {
            res = callback.call(());
        });
        res
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

    /// Enumerable own keys of the global object.
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

    /// Run one pending microtask/job.
    #[napi(js_name = "execute_pending_job")]
    pub fn execute_pending_job(&self) -> napi::Result<bool> {
        Ok(self.inner.with(|ctx| ctx.execute_pending_job()))
    }

    /// Whether a JS exception is currently pending in this context.
    #[napi(js_name = "has_exception")]
    pub fn has_exception(&self) -> napi::Result<bool> {
        Ok(self.inner.with(|ctx| ctx.has_exception()))
    }

    /// Return the pending JS exception marshalled into a JS value.
    #[napi]
    pub fn catch(&self) -> napi::Result<JsonValue> {
        self.inner.with(|ctx| {
            let value = ctx.catch();
            value_to_json(&ctx, value)
        })
    }
}

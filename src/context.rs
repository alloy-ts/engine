use napi::bindgen_prelude::{Function, Unknown};
use napi::Error;
use napi_derive::napi;
use serde_json::Value as JsonValue;

use rquickjs::{
    context::EvalOptions, Ctx, FromJs, Runtime as QjsRuntime, Context as QjsContext, Type, Value,
};

use crate::engine::JsEngine;
use crate::runtime::JsRuntime;

/// Map a [`rquickjs::Error`] into a `napi::Error` so JS sees a thrown exception.
fn to_napi_err(e: rquickjs::Error) -> Error {
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
    pub(crate) inner: QjsContext,
}

#[napi]
impl JsContext {
    /// Create a `Context` backed by a freshly-created *private* QuickJS runtime.
    #[napi(factory)]
    pub fn new() -> napi::Result<Self> {
        let runtime = QjsRuntime::new().map_err(to_napi_err)?;
        let inner = QjsContext::full(&runtime).map_err(to_napi_err)?;
        Ok(JsContext { inner })
    }

    /// Context.fromRaw() / Context.from_raw()
    #[napi(factory, js_name = "fromRaw")]
    pub fn from_raw_camel(engine: &JsEngine) -> napi::Result<Self> {
        let inner = QjsContext::full(&engine.runtime).map_err(to_napi_err)?;
        Ok(JsContext { inner })
    }

    #[napi(factory, js_name = "from_raw")]
    pub fn from_raw(engine: &JsEngine) -> napi::Result<Self> {
        Self::from_raw_camel(engine)
    }

    /// Context.base()
    #[napi(factory)]
    pub fn base(engine: Option<&JsEngine>) -> napi::Result<Self> {
        let runtime = match engine {
            Some(e) => e.runtime.clone(),
            None => QjsRuntime::new().map_err(to_napi_err)?,
        };
        let inner = QjsContext::base(&runtime).map_err(to_napi_err)?;
        Ok(JsContext { inner })
    }

    /// Context.custom()
    #[napi(factory)]
    pub fn custom(engine: Option<&JsEngine>) -> napi::Result<Self> {
        let runtime = match engine {
            Some(e) => e.runtime.clone(),
            None => QjsRuntime::new().map_err(to_napi_err)?,
        };
        let inner = QjsContext::base(&runtime).map_err(to_napi_err)?;
        Ok(JsContext { inner })
    }

    /// Context.full()
    #[napi(factory)]
    pub fn full(engine: Option<&JsEngine>) -> napi::Result<Self> {
        let runtime = match engine {
            Some(e) => e.runtime.clone(),
            None => QjsRuntime::new().map_err(to_napi_err)?,
        };
        let inner = QjsContext::full(&runtime).map_err(to_napi_err)?;
        Ok(JsContext { inner })
    }

    /// Context.builder()
    #[napi(factory)]
    pub fn builder() -> napi::Result<Self> {
        Self::new()
    }

    pub(crate) fn from_inner(inner: QjsContext) -> Self {
        JsContext { inner }
    }

    /// Context.prototype.asRaw() / Context.prototype.as_raw()
    #[napi(js_name = "asRaw")]
    pub fn as_raw_camel(&self) -> String {
        format!("{:p}", self.inner.as_raw().as_ptr())
    }

    #[napi(js_name = "as_raw")]
    pub fn as_raw(&self) -> String {
        self.as_raw_camel()
    }

    /// Context.prototype.getEnginePtr() / Context.prototype.get_engine_ptr()
    #[napi(js_name = "getEnginePtr")]
    pub fn get_engine_ptr_camel(&self) -> String {
        format!("{:p}", self.inner.get_runtime_ptr())
    }

    #[napi(js_name = "get_engine_ptr")]
    pub fn get_engine_ptr(&self) -> String {
        self.get_engine_ptr_camel()
    }

    /// Context.prototype.Engine() / Context.prototype.engine()
    #[napi(js_name = "Engine")]
    pub fn engine_capital(&self) -> JsRuntime {
        JsRuntime::from_inner(self.inner.runtime().clone())
    }

    #[napi(js_name = "engine")]
    pub fn engine(&self) -> JsRuntime {
        self.engine_capital()
    }

    /// Context.prototype.with(callback)
    #[napi]
    pub fn with<'env>(&self, callback: Function<'env, (), Unknown<'env>>) -> napi::Result<Unknown<'env>> {
        self.inner.with(|_ctx| callback.call(()))
    }

    /// Evaluate `source` as a global script and return the result as a JS value.
    #[napi]
    pub fn eval(&self, source: String) -> napi::Result<JsonValue> {
        self.inner.with(|ctx| {
            let value: Value = ctx.eval(source).map_err(to_napi_err)?;
            value_to_json(&ctx, value)
        })
    }

    /// Evaluate `source` with a specified filename/path for stack trace reporting.
    #[napi(js_name = "eval_with_filename")]
    pub fn eval_with_filename(&self, source: String, filename: String) -> napi::Result<JsonValue> {
        self.inner.with(|ctx| {
            let mut opts = EvalOptions::default();
            opts.filename = Some(filename);
            let value: Value = ctx.eval_with_options(source, opts).map_err(to_napi_err)?;
            value_to_json(&ctx, value)
        })
    }

    /// Evaluate a script with compile-only flag or validate options.
    #[napi]
    pub fn compile(&self, source: String, filename: String, strict: Option<bool>) -> napi::Result<()> {
        self.inner.with(|ctx| {
            let mut opts = EvalOptions::default();
            opts.filename = Some(filename);
            opts.strict = strict.unwrap_or(true);
            let _: Value = ctx.eval_with_options(source, opts).map_err(to_napi_err)?;
            Ok(())
        })
    }

    /// Evaluate `source` as an ES module (`import` / `export` allowed) and return
    /// its namespace object as a JS value.
    #[napi(js_name = "eval_module")]
    pub fn eval_module(&self, source: String) -> napi::Result<JsonValue> {
        self.inner.with(|ctx| {
            let (module, promise) =
                rquickjs::Module::declare(ctx.clone(), "eval_module", source)
                    .map_err(to_napi_err)?
                    .eval()
                    .map_err(to_napi_err)?;
            let _ = promise.finish::<()>();
            let ns = module.namespace().map_err(to_napi_err)?;
            let obj = rquickjs::Object::new(ctx.clone()).map_err(to_napi_err)?;
            for key in ns.keys::<rquickjs::String>() {
                if let Ok(k) = key {
                    if let Ok(v) = ns.get::<_, Value>(k.clone()) {
                        let _ = obj.set(k, v);
                    }
                }
            }
            value_to_json(&ctx, Value::from(obj))
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

    /// Own property names of the global object (`Object.getOwnPropertyNames(globalThis)`).
    #[napi(js_name = "global_keys")]
    pub fn global_keys(&self) -> napi::Result<Vec<String>> {
        self.inner.with(|ctx| {
            let keys: Vec<String> = ctx
                .eval("Object.getOwnPropertyNames(globalThis)")
                .map_err(to_napi_err)?;
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

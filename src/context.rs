use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde_json::Value as JsonValue;
use std::sync::atomic::{AtomicU64, Ordering};

use rquickjs::{
    Ctx, FromJs, Module, Runtime as QjsRuntime, Context as QjsContext, Type, Value,
};

use crate::engine::JsEngine;

/// Monotonic counter used to give each `eval_module` call a unique module name,
/// since QuickJS keys its module registry by name (re-declaring a name errors).
static MODULE_SEQ: AtomicU64 = AtomicU64::new(0);

/// Map a [`rquickjs::Error`] into a `napi::Error` so JS sees a thrown exception.
fn to_napi_err(e: rquickjs::Error) -> Error {
    Error::from_reason(e.to_string())
}

/// Convert a QuickJS value into a JSON value Node can receive directly.
///
/// Data types (objects, arrays, numbers, strings, booleans, null) round-trip
/// losslessly. Functions / symbols / promises are not JSON-representable and
/// collapse to `null` for now (see module docs).
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
    pub(crate) inner: QjsContext,
}

#[napi]
impl JsContext {
    /// Create a `Context` backed by a freshly-created *private* QuickJS runtime.
    ///
    /// This is the self-contained form. Prefer `new Runtime().create_context()`
    /// to share one engine across several contexts.
    #[napi(constructor)]
    pub fn new() -> napi::Result<Self> {
        let runtime = QjsRuntime::new().map_err(to_napi_err)?;
        let inner = QjsContext::full(&runtime).map_err(to_napi_err)?;
        Ok(JsContext { inner })
    }

    pub(crate) fn from_inner(inner: QjsContext) -> Self {
        JsContext { inner }
    }

    /// Create a Context from a raw context pointer and a JsEngine.
    #[napi(factory, js_name = "from_raw")]
    pub fn from_raw(engine: &JsEngine, ptr: BigInt) -> napi::Result<Self> {
        let (signed, raw_ptr, lossless) = ptr.get_u64();
        if signed || !lossless || raw_ptr == 0 {
            return Err(Error::from_reason("Invalid raw context pointer"));
        }
        let non_null = std::ptr::NonNull::new(raw_ptr as *mut rquickjs::qjs::JSContext)
            .ok_or_else(|| Error::from_reason("Null pointer"))?;
        unsafe { rquickjs::qjs::JS_DupContext(non_null.as_ptr()) };
        let inner = unsafe { QjsContext::from_raw(non_null, engine.inner().clone()) };
        Ok(JsContext { inner })
    }

    /// Alias for `from_raw` in camelCase for JS convenience.
    #[napi(factory, js_name = "fromRaw")]
    pub fn from_raw_camel(engine: &JsEngine, ptr: BigInt) -> napi::Result<Self> {
        Self::from_raw(engine, ptr)
    }

    /// Creates a base context with only required functions registered.
    #[napi(factory, js_name = "base")]
    pub fn base(engine: &JsEngine) -> napi::Result<Self> {
        let inner = QjsContext::base(engine.inner()).map_err(to_napi_err)?;
        Ok(JsContext { inner })
    }

    /// Creates a context with custom intrinsics registered.
    #[napi(factory, js_name = "custom")]
    pub fn custom(engine: &JsEngine) -> napi::Result<Self> {
        let inner = QjsContext::custom::<rquickjs::context::intrinsic::None>(engine.inner())
            .map_err(to_napi_err)?;
        Ok(JsContext { inner })
    }

    /// Creates a context with all standard intrinsics registered.
    #[napi(factory, js_name = "full")]
    pub fn full(engine: &JsEngine) -> napi::Result<Self> {
        let inner = QjsContext::full(engine.inner()).map_err(to_napi_err)?;
        Ok(JsContext { inner })
    }

    /// Create a context builder for creating a context with a specific set of intrinsics.
    #[napi(factory, js_name = "builder")]
    pub fn builder(engine: &JsEngine) -> napi::Result<Self> {
        let inner = QjsContext::builder()
            .with::<rquickjs::context::intrinsic::Eval>()
            .build(engine.inner())
            .map_err(to_napi_err)?;
        Ok(JsContext { inner })
    }

    /// Returns the raw pointer to the QuickJS C context as a BigInt.
    #[napi(js_name = "as_raw")]
    pub fn as_raw(&self) -> BigInt {
        BigInt::from(self.inner.as_raw().as_ptr() as usize as u64)
    }

    /// Alias for `as_raw` in camelCase for JS convenience.
    #[napi(js_name = "asRaw")]
    pub fn as_raw_camel(&self) -> BigInt {
        self.as_raw()
    }

    /// Returns the pointer to the underlying QuickJS Engine (JSRuntime) as a BigInt.
    #[napi(js_name = "get_engine_ptr")]
    pub fn get_engine_ptr(&self) -> BigInt {
        let ptr = unsafe { rquickjs::qjs::JS_GetRuntime(self.inner.as_raw().as_ptr()) };
        BigInt::from(ptr as usize as u64)
    }

    /// Alias for `get_engine_ptr` in camelCase for JS convenience.
    #[napi(js_name = "getEnginePtr")]
    pub fn get_engine_ptr_camel(&self) -> BigInt {
        self.get_engine_ptr()
    }

    /// Returns the associated Engine wrapper instance.
    #[napi(js_name = "engine")]
    pub fn engine(&self, env: Env) -> napi::Result<JsEngine> {
        Ok(JsEngine::from_inner(self.inner.runtime().clone(), env))
    }

    /// Alias for `engine` in PascalCase for JS convenience.
    #[napi(js_name = "Engine")]
    pub fn engine_pascal(&self, env: Env) -> napi::Result<JsEngine> {
        self.engine(env)
    }

    /// Executes a callback inside this context's scope and returns its result.
    #[napi(js_name = "with")]
    pub fn with_callback<'env>(&self, env: &'env Env, callback: Function<'env, (), Unknown<'env>>) -> napi::Result<Unknown<'env>> {
        self.inner.with(|_c| {
            let global = env.get_global()?;
            callback.apply(global, ())
        })
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
    ///
    /// Returns `true` if a job actually ran. Call in a loop to drain the queue.
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

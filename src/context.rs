use std::ptr::NonNull;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use napi::bindgen_prelude::{BigInt, FromNapiValue, Function, ToNapiValue, Unknown};
use napi::{Env, Error};
use napi_derive::napi;
use rquickjs::context::intrinsic;
use rquickjs::{
    qjs, Context as Inner, Ctx, FromJs, Module, Runtime, Type, Value,
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

/// NAPI class wrapping an embedded QuickJS [`Context`](rquickjs::Context).
///
/// JS:
/// ```ts
/// const ctx = new Context();
/// const v = ctx.eval("1 + 1"); // 2
/// ```
#[napi]
pub struct Context {
    /// The embedded QuickJS context. It keeps its runtime alive for us.
    inner: Inner,
    /// The host N-API environment, captured at construction.
    env: Env,
    /// Set to `true` when the owning addon (and thus this context) is unloaded.
    alive: Arc<AtomicBool>,
}

#[napi]
impl Context {
    /// Create a `Context` backed by a freshly-created *private* QuickJS runtime.
    #[napi(constructor)]
    pub fn new(env: Env) -> napi::Result<Self> {
        let runtime = Runtime::new().map_err(to_napi_err)?;
        let inner = Inner::full(&runtime).map_err(to_napi_err)?;
        Self::create_standalone(inner, env)
    }

    fn create_standalone(inner: Inner, env: Env) -> napi::Result<Self> {
        let alive = Arc::new(AtomicBool::new(false));
        let hook_data = Arc::clone(&alive);
        env.add_env_cleanup_hook(hook_data, |flag: Arc<AtomicBool>| {
            flag.store(true, Ordering::SeqCst);
        })?;
        Ok(Context { inner, env, alive })
    }

    pub(crate) fn clone_inner(&self) -> Inner {
        self.inner.clone()
    }

    // --- Static constructor methods ---

    #[napi(js_name = "fromRaw")]
    pub fn from_raw(env: Env, ctx_ptr: BigInt, engine: &JsEngine) -> napi::Result<Self> {
        let (negative, value, lossless) = ctx_ptr.get_u64();
        if negative || !lossless {
            return Err(Error::from_reason("context pointer does not fit in a u64"));
        }
        let ptr = value as *mut qjs::JSContext;
        let nn = NonNull::new(ptr)
            .ok_or_else(|| Error::from_reason("context pointer is null"))?;
        let inner = unsafe { Inner::from_raw(nn, engine.get_runtime().clone()) };
        Self::create_standalone(inner, env)
    }

    #[napi]
    pub fn base(env: Env, engine: Option<&JsEngine>) -> napi::Result<Self> {
        let inner = if let Some(e) = engine {
            Inner::base(e.get_runtime()).map_err(to_napi_err)?
        } else {
            let runtime = Runtime::new().map_err(to_napi_err)?;
            Inner::base(&runtime).map_err(to_napi_err)?
        };
        Self::create_standalone(inner, env)
    }

    #[napi]
    pub fn custom(
        env: Env,
        engine: Option<&JsEngine>,
        intrinsics: Option<Vec<String>>,
    ) -> napi::Result<Self> {
        let runtime_owned;
        let rt = if let Some(e) = engine {
            e.get_runtime()
        } else {
            runtime_owned = Runtime::new().map_err(to_napi_err)?;
            &runtime_owned
        };

        if let Some(ref list) = intrinsics {
            if list.iter().any(|s| s.eq_ignore_ascii_case("all")) {
                let inner = Inner::full(rt).map_err(to_napi_err)?;
                return Self::create_standalone(inner, env);
            } else if list.is_empty() || list.iter().any(|s| s.eq_ignore_ascii_case("none")) {
                let inner = Inner::custom::<intrinsic::None>(rt).map_err(to_napi_err)?;
                return Self::create_standalone(inner, env);
            } else {
                let mut builder = ContextBuilder::new();
                for name in list {
                    builder.with(name.clone());
                }
                return builder.build(env, engine);
            }
        } else {
            let inner = Inner::custom::<intrinsic::None>(rt).map_err(to_napi_err)?;
            Self::create_standalone(inner, env)
        }
    }

    #[napi]
    pub fn full(env: Env, engine: Option<&JsEngine>) -> napi::Result<Self> {
        let inner = if let Some(e) = engine {
            Inner::full(e.get_runtime()).map_err(to_napi_err)?
        } else {
            let runtime = Runtime::new().map_err(to_napi_err)?;
            Inner::full(&runtime).map_err(to_napi_err)?
        };
        Self::create_standalone(inner, env)
    }

    #[napi]
    pub fn builder() -> ContextBuilder {
        ContextBuilder::new()
    }

    // --- Instance prototype methods ---

    #[napi(js_name = "asRaw")]
    pub fn as_raw(&self) -> napi::Result<BigInt> {
        let ptr = self.inner.as_raw().as_ptr() as u64;
        Ok(BigInt::from(ptr))
    }

    #[napi(js_name = "getEnginePtr")]
    pub fn get_engine_ptr(&self) -> napi::Result<BigInt> {
        let ptr = unsafe { qjs::JS_GetRuntime(self.inner.as_raw().as_ptr()) } as u64;
        Ok(BigInt::from(ptr))
    }

    #[napi(js_name = "Engine")]
    pub fn engine(&self) -> JsEngine {
        JsEngine::from_runtime(
            self.inner.runtime().clone(),
            self.env,
            Arc::clone(&self.alive),
        )
    }

    #[napi]
    pub fn with<'a>(&self, callback: Function<'a>) -> napi::Result<Unknown<'a>> {
        self.inner.with(|ctx| {
            let ctx_static = unsafe { std::mem::transmute::<Ctx<'_>, Ctx<'static>>(ctx) };
            let jsctx = JsCtx::from_inner(ctx_static);
            let raw = unsafe { JsCtx::to_napi_value(self.env.raw(), jsctx)? };
            let unknown = unsafe { Unknown::from_napi_value(self.env.raw(), raw)? };
            callback.call(unknown)
        })
    }

    // --- Additional Context evaluation helpers ---

    #[napi]
    pub fn eval(&self, source: String) -> napi::Result<JsonValue> {
        self.inner.with(|ctx| {
            let value: Value = ctx.eval(source).map_err(to_napi_err)?;
            value_to_json(&ctx, value)
        })
    }

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

    #[napi]
    pub fn json_parse(&self, json: String) -> napi::Result<JsonValue> {
        self.inner.with(|ctx| {
            let value = ctx.json_parse(json).map_err(to_napi_err)?;
            value_to_json(&ctx, value)
        })
    }

    #[napi]
    pub fn global_keys(&self) -> napi::Result<Vec<String>> {
        self.inner.with(|ctx| {
            let keys: Vec<String> = ctx.eval("Object.keys(this)").map_err(to_napi_err)?;
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
            value_to_json(&ctx, value)
        })
    }
}

#[napi]
pub struct ContextBuilder {
    eval: bool,
    promise: bool,
    bignum: bool,
    json: bool,
    proxy: bool,
    typed_arrays: bool,
    map_set: bool,
    regexp: bool,
    date: bool,
}

#[napi]
impl ContextBuilder {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            eval: false,
            promise: false,
            bignum: false,
            json: false,
            proxy: false,
            typed_arrays: false,
            map_set: false,
            regexp: false,
            date: false,
        }
    }

    #[napi]
    pub fn with_eval(&mut self) -> &Self {
        self.eval = true;
        self
    }

    #[napi]
    pub fn with_promise(&mut self) -> &Self {
        self.promise = true;
        self
    }

    #[napi]
    pub fn with_bignum(&mut self) -> &Self {
        self.bignum = true;
        self
    }

    #[napi]
    pub fn with_json(&mut self) -> &Self {
        self.json = true;
        self
    }

    #[napi]
    pub fn with_proxy(&mut self) -> &Self {
        self.proxy = true;
        self
    }

    #[napi]
    pub fn with_typed_arrays(&mut self) -> &Self {
        self.typed_arrays = true;
        self
    }

    #[napi]
    pub fn with_map_set(&mut self) -> &Self {
        self.map_set = true;
        self
    }

    #[napi]
    pub fn with_regexp(&mut self) -> &Self {
        self.regexp = true;
        self
    }

    #[napi]
    pub fn with_date(&mut self) -> &Self {
        self.date = true;
        self
    }

    #[napi]
    pub fn with(&mut self, intrinsic: String) -> &Self {
        match intrinsic.to_lowercase().as_str() {
            "eval" | "intrinsic::eval" => self.eval = true,
            "promise" | "intrinsic::promise" => self.promise = true,
            "bignum" | "intrinsic::bignum" => self.bignum = true,
            "json" | "intrinsic::json" => self.json = true,
            "proxy" | "intrinsic::proxy" => self.proxy = true,
            "typedarrays" | "typed_arrays" | "intrinsic::typedarrays" => self.typed_arrays = true,
            "mapset" | "map_set" | "intrinsic::mapset" => self.map_set = true,
            "regexp" | "intrinsic::regexp" => self.regexp = true,
            "date" | "intrinsic::date" => self.date = true,
            _ => {}
        }
        self
    }

    #[napi]
    pub fn build(&self, env: Env, engine: Option<&JsEngine>) -> napi::Result<Context> {
        let runtime_owned;
        let rt = if let Some(e) = engine {
            e.get_runtime()
        } else {
            runtime_owned = Runtime::new().map_err(to_napi_err)?;
            &runtime_owned
        };

        let inner = Inner::custom::<intrinsic::None>(rt).map_err(to_napi_err)?;
        inner.with(|ctx| unsafe {
            let ptr = ctx.as_raw().as_ptr();
            if self.eval {
                qjs::JS_AddIntrinsicEval(ptr);
            }
            if self.promise {
                qjs::JS_AddIntrinsicPromise(ptr);
            }
            if self.bignum {
                qjs::JS_AddIntrinsicBigInt(ptr);
            }
            if self.json {
                qjs::JS_AddIntrinsicJSON(ptr);
            }
            if self.proxy {
                qjs::JS_AddIntrinsicProxy(ptr);
            }
            if self.typed_arrays {
                qjs::JS_AddIntrinsicTypedArrays(ptr);
            }
            if self.map_set {
                qjs::JS_AddIntrinsicMapSet(ptr);
            }
            if self.regexp {
                qjs::JS_AddIntrinsicRegExp(ptr);
            }
            if self.date {
                qjs::JS_AddIntrinsicDate(ptr);
            }
        });

        Context::create_standalone(inner, env)
    }
}

#[napi]
pub struct JsCtx {
    inner: Ctx<'static>,
}

#[napi]
impl JsCtx {
    pub(crate) fn from_inner(inner: Ctx<'static>) -> Self {
        JsCtx { inner }
    }

    #[napi]
    pub fn eval(&self, source: String) -> napi::Result<JsonValue> {
        let value: Value = self.inner.eval(source).map_err(to_napi_err)?;
        value_to_json(&self.inner, value)
    }

    #[napi]
    pub fn eval_module(&self, source: String) -> napi::Result<JsonValue> {
        let seq = MODULE_SEQ.fetch_add(1, Ordering::Relaxed);
        let name = format!("<eval:{seq}>");
        let module = Module::declare(self.inner.clone(), name, source).map_err(to_napi_err)?;
        let (evaluated, _promise) = module.eval().map_err(to_napi_err)?;
        let ns = evaluated.namespace().map_err(to_napi_err)?;
        value_to_json(&self.inner, ns.into_value())
    }

    #[napi]
    pub fn json_parse(&self, json: String) -> napi::Result<JsonValue> {
        let value = self.inner.json_parse(json).map_err(to_napi_err)?;
        value_to_json(&self.inner, value)
    }

    #[napi]
    pub fn global_keys(&self) -> napi::Result<Vec<String>> {
        let keys: Vec<String> = self.inner.eval("Object.keys(this)").map_err(to_napi_err)?;
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
        value_to_json(&self.inner, value)
    }
}

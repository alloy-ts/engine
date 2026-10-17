//! NAPI bindings for `rquickjs::Context`.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rquickjs::{Context as QuickContext, Runtime as QuickRuntime};

use crate::engine::JsEngine;

/// NAPI wrapper around `rquickjs::Context`.
#[napi(js_name = "Context")]
pub struct Context {
    pub(crate) inner: QuickContext,
}

#[napi]
impl Context {
    /// Static constructor: Context.fromRaw()
    #[napi(factory, js_name = "fromRaw")]
    pub fn from_raw(engine: &JsEngine, ptr: BigInt) -> Result<Self> {
        let (_, val, _) = ptr.get_u64();
        let raw_ptr = val as *mut rquickjs::qjs::JSContext;
        let non_null = std::ptr::NonNull::new(raw_ptr).ok_or_else(|| {
            Error::new(Status::InvalidArg, "Null context pointer passed to fromRaw")
        })?;
        let inner = unsafe { QuickContext::from_raw(non_null, engine.runtime.clone()) };
        Ok(Self { inner })
    }

    /// Static constructor: Context.base()
    #[napi(factory, js_name = "base")]
    pub fn base(engine: &JsEngine) -> Result<Self> {
        let inner = QuickContext::base(&engine.runtime).map_err(|e| {
            Error::new(Status::GenericFailure, format!("Failed to create QuickJS base context: {e}"))
        })?;
        Ok(Self { inner })
    }

    /// Static constructor: Context.custom()
    #[napi(factory, js_name = "custom")]
    pub fn custom(engine: &JsEngine) -> Result<Self> {
        let inner = QuickContext::custom::<rquickjs::context::intrinsic::None>(&engine.runtime).map_err(|e| {
            Error::new(Status::GenericFailure, format!("Failed to create QuickJS custom context: {e}"))
        })?;
        Ok(Self { inner })
    }

    /// Static constructor: Context.full()
    #[napi(factory, js_name = "full")]
    pub fn full(engine: &JsEngine) -> Result<Self> {
        let inner = QuickContext::full(&engine.runtime).map_err(|e| {
            Error::new(Status::GenericFailure, format!("Failed to create QuickJS context: {e}"))
        })?;
        Ok(Self { inner })
    }

    /// Static constructor: Context.builder()
    #[napi(factory, js_name = "builder")]
    pub fn builder(engine: &JsEngine) -> Result<Self> {
        let inner = QuickContext::full(&engine.runtime).map_err(|e| {
            Error::new(Status::GenericFailure, format!("Failed to build QuickJS context: {e}"))
        })?;
        Ok(Self { inner })
    }

    /// Prototype method: context.asRaw()
    #[napi(js_name = "asRaw")]
    pub fn as_raw(&self) -> BigInt {
        BigInt {
            sign_bit: false,
            words: vec![self.inner.as_raw().as_ptr() as u64],
        }
    }

    /// Prototype method: context.getEnginePtr()
    #[napi(js_name = "getEnginePtr")]
    pub fn get_engine_ptr(&self) -> BigInt {
        BigInt {
            sign_bit: false,
            words: vec![self.inner.get_runtime_ptr() as u64],
        }
    }

    /// Prototype method / getter: context.Engine()
    #[napi(js_name = "Engine")]
    pub fn engine(&self, env: Env) -> Result<JsEngine> {
        JsEngine::new(env)
    }

    /// Prototype method: context.with(cb)
    #[napi(js_name = "with")]
    pub fn with_callback(&self, callback: Function<(), String>) -> Result<String> {
        self.inner.with(|_ctx| {
            callback.call(())
        })
    }

    /// Evaluate a JavaScript string in this context and return the result as a JSON string or string representation.
    #[napi(js_name = "eval")]
    pub fn eval(&self, script: String) -> Result<String> {
        self.inner.with(|ctx| {
            let res: rquickjs::Value = ctx.eval(script).map_err(|e| {
                Error::new(Status::GenericFailure, format!("QuickJS Eval Error: {e}"))
            })?;
            if res.is_string() {
                let s: String = res.into_string().unwrap().to_string().map_err(|e| {
                    Error::new(Status::GenericFailure, format!("Failed to convert string: {e}"))
                })?;
                Ok(s)
            } else {
                let json: Option<rquickjs::String> = ctx.json_stringify(res).map_err(|e| {
                    Error::new(Status::GenericFailure, format!("JSON stringify error: {e}"))
                })?;
                match json {
                    Some(s) => s.to_string().map_err(|e| {
                        Error::new(Status::GenericFailure, format!("Failed to stringify: {e}"))
                    }),
                    None => Ok("undefined".to_string()),
                }
            }
        })
    }
}

impl Context {
    pub fn full_from_quick_runtime(runtime: &QuickRuntime) -> Result<Self> {
        let inner = QuickContext::full(runtime).map_err(|e| {
            Error::new(Status::GenericFailure, format!("Failed to create QuickJS context: {e}"))
        })?;
        Ok(Self { inner })
    }

    pub fn base_from_quick_runtime(runtime: &QuickRuntime) -> Result<Self> {
        let inner = QuickContext::base(runtime).map_err(|e| {
            Error::new(Status::GenericFailure, format!("Failed to create QuickJS base context: {e}"))
        })?;
        Ok(Self { inner })
    }
}

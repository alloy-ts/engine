//! NAPI bindings for `rquickjs::Context`.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rquickjs::{Context as QuickContext, Runtime as QuickRuntime};

/// NAPI wrapper around `rquickjs::Context`.
#[napi(js_name = "Context")]
pub struct Context {
    pub(crate) inner: QuickContext,
}

#[napi]
impl Context {
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
    pub fn full(runtime: &QuickRuntime) -> Result<Self> {
        let inner = QuickContext::full(runtime).map_err(|e| {
            Error::new(Status::GenericFailure, format!("Failed to create QuickJS context: {e}"))
        })?;
        Ok(Self { inner })
    }

    pub fn base(runtime: &QuickRuntime) -> Result<Self> {
        let inner = QuickContext::base(runtime).map_err(|e| {
            Error::new(Status::GenericFailure, format!("Failed to create QuickJS base context: {e}"))
        })?;
        Ok(Self { inner })
    }
}

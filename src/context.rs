//! NAPI bindings for QuickJS [`Context`].

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rquickjs::{Context as QjsContext, Runtime as QjsRuntime};

/// JS class wrapping a QuickJS execution [`Context`].
#[napi(js_name = "Context")]
pub struct Context {
    pub(crate) inner: QjsContext,
}

#[napi]
impl Context {
    /// Create a new standard QuickJS execution context.
    #[napi(constructor)]
    pub fn new() -> Result<Self> {
        let rt = QjsRuntime::new().map_err(|e| Error::from_reason(e.to_string()))?;
        let ctx = QjsContext::full(&rt).map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(Self { inner: ctx })
    }

    /// Create a base context with minimal intrinsics registered.
    #[napi(factory)]
    pub fn create_base() -> Result<Self> {
        let rt = QjsRuntime::new().map_err(|e| Error::from_reason(e.to_string()))?;
        let ctx = QjsContext::base(&rt).map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(Self { inner: ctx })
    }

    /// Evaluate JavaScript code within this Context and return the stringified result.
    #[napi]
    pub fn eval(&self, code: String) -> Result<String> {
        self.inner.with(|ctx| {
            let val: rquickjs::Value = ctx.eval(code).map_err(|e| Error::from_reason(e.to_string()))?;
            if val.is_undefined() {
                Ok("undefined".to_string())
            } else if val.is_null() {
                Ok("null".to_string())
            } else if let Some(s) = val.as_string() {
                s.to_string().map_err(|e| Error::from_reason(e.to_string()))
            } else if let Some(b) = val.as_bool() {
                Ok(b.to_string())
            } else if let Some(i) = val.as_int() {
                Ok(i.to_string())
            } else if let Some(f) = val.as_float() {
                Ok(f.to_string())
            } else if let Ok(Some(json)) = ctx.json_stringify(&val) {
                json.to_string().map_err(|e| Error::from_reason(e.to_string()))
            } else {
                Ok("[object Object]".to_string())
            }
        })
    }
}

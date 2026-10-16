//! NAPI bindings for QuickJS Context (`rquickjs::Context`).

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rquickjs::{Context as QjsContext, Runtime as QjsRuntime, Value as QjsValue};

#[napi]
pub struct Context {
  pub(crate) inner: QjsContext,
}

#[napi]
impl Context {
  /// Create a new full QuickJS context.
  #[napi(constructor)]
  pub fn new() -> Result<Self> {
    let rt = QjsRuntime::new().map_err(|e| Error::from_reason(e.to_string()))?;
    let inner = QjsContext::full(&rt).map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(Self { inner })
  }

  /// Create a full context with all standard intrinsics.
  #[napi(factory, js_name = "full")]
  pub fn full() -> Result<Self> {
    Self::new()
  }

  /// Create a base context with minimal intrinsics.
  #[napi(factory, js_name = "base")]
  pub fn base() -> Result<Self> {
    let rt = QjsRuntime::new().map_err(|e| Error::from_reason(e.to_string()))?;
    let inner = QjsContext::base(&rt).map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(Self { inner })
  }

  /// Evaluate a JavaScript script within this context and return the stringified result.
  #[napi(js_name = "eval")]
  pub fn eval(&self, script: String) -> Result<String> {
    self.inner.with(|ctx| {
      let val: QjsValue = ctx.eval(script).map_err(|e| Error::from_reason(e.to_string()))?;
      if val.is_undefined() {
        Ok("undefined".to_string())
      } else if val.is_null() {
        Ok("null".to_string())
      } else if let Some(s) = val.as_string() {
        s.to_string().map_err(|e| Error::from_reason(e.to_string()))
      } else if let Some(i) = val.as_int() {
        Ok(i.to_string())
      } else if let Some(f) = val.as_float() {
        Ok(f.to_string())
      } else if let Some(b) = val.as_bool() {
        Ok(b.to_string())
      } else {
        let json: Option<rquickjs::String> = ctx
          .json_stringify(val)
          .map_err(|e| Error::from_reason(e.to_string()))?;
        match json {
          Some(s) => s.to_string().map_err(|e| Error::from_reason(e.to_string())),
          None => Ok("[object Object]".to_string()),
        }
      }
    })
  }

  /// Evaluate a JavaScript script and return integer result.
  #[napi(js_name = "eval_int")]
  pub fn eval_int(&self, script: String) -> Result<i32> {
    self.inner.with(|ctx| {
      ctx.eval::<i32, _>(script).map_err(|e| Error::from_reason(e.to_string()))
    })
  }

  /// Evaluate a JavaScript script and return boolean result.
  #[napi(js_name = "eval_bool")]
  pub fn eval_bool(&self, script: String) -> Result<bool> {
    self.inner.with(|ctx| {
      ctx.eval::<bool, _>(script).map_err(|e| Error::from_reason(e.to_string()))
    })
  }
}

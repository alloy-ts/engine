//! NAPI bindings for QuickJS `Context`.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rquickjs::{Context as QjsContext, Runtime as QjsRuntime};

#[napi]
pub struct Context {
  pub(crate) inner: QjsContext,
}

#[napi]
impl Context {
  /// Creates a base context with only the required functions registered.
  #[napi(factory)]
  pub fn base() -> Result<Self> {
    let rt = QjsRuntime::new().map_err(|e| Error::from_reason(e.to_string()))?;
    let ctx = QjsContext::base(&rt).map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(Self { inner: ctx })
  }

  /// Creates a context with all standard available intrinsics registered.
  #[napi(factory)]
  pub fn full() -> Result<Self> {
    let rt = QjsRuntime::new().map_err(|e| Error::from_reason(e.to_string()))?;
    let ctx = QjsContext::full(&rt).map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(Self { inner: ctx })
  }

  /// Evaluate a JavaScript expression inside this context and return the result.
  #[napi]
  pub fn eval(&self, code: String) -> Result<String> {
    self.inner.with(|ctx| {
      let val: rquickjs::Value = ctx.eval(code).map_err(|e| Error::from_reason(e.to_string()))?;
      if let Some(s) = val.as_string() {
        s.to_string().map_err(|e| Error::from_reason(e.to_string()))
      } else if let Some(i) = val.as_int() {
        Ok(i.to_string())
      } else if let Some(f) = val.as_float() {
        Ok(f.to_string())
      } else if let Some(b) = val.as_bool() {
        Ok(b.to_string())
      } else {
        Ok(format!("{:?}", val))
      }
    })
  }
}

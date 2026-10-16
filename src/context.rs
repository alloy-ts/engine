//! NAPI bindings for QuickJS Context.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rquickjs::{Context as QuickContext, Runtime as QuickRuntime};

/// NAPI JS wrapper around QuickJS Context.
#[napi]
pub struct Context {
  inner: QuickContext,
}

#[napi]
impl Context {
  /// Create a new QuickJS execution context.
  #[napi(constructor)]
  pub fn new() -> Result<Self> {
    let rt = QuickRuntime::new().map_err(|e| Error::from_reason(e.to_string()))?;
    let ctx = QuickContext::full(&rt).map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(Context { inner: ctx })
  }

  /// Evaluate a JavaScript script in this context and return the result as a string.
  #[napi]
  pub fn eval(&self, code: String) -> Result<String> {
    self.inner.with(|ctx| {
      let res: String = ctx
        .eval(code)
        .map_err(|e| Error::from_reason(e.to_string()))?;
      Ok(res)
    })
  }
}

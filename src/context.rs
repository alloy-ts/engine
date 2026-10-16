//! NAPI bindings for QuickJS Context (`rquickjs::Context`).

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rquickjs::{Context as QuickJSContext, Runtime as QuickJSRuntime};

/// NAPI wrapper around `rquickjs::Context`.
#[napi]
pub struct Context {
  pub(crate) inner: QuickJSContext,
}

#[napi]
impl Context {
  /// Create a new full Context from a QuickJS Runtime or a newly initialized one.
  #[napi(constructor)]
  pub fn new() -> Result<Self> {
    let rt = QuickJSRuntime::new().map_err(|e| Error::from_reason(e.to_string()))?;
    let ctx = QuickJSContext::full(&rt).map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(Self { inner: ctx })
  }

  /// Evaluate a JavaScript script within the context and return stringified result or error.
  #[napi]
  pub fn eval(&self, script: String) -> Result<String> {
    self.inner.with(|ctx| {
      let val: rquickjs::Value = ctx.eval(script).map_err(|e| Error::from_reason(e.to_string()))?;
      let json_str = ctx
        .json_stringify(val)
        .map_err(|e| Error::from_reason(e.to_string()))?
        .map(|s| s.to_string().unwrap_or_default())
        .unwrap_or_else(|| "undefined".to_string());
      Ok(json_str)
    })
  }
}

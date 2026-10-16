//! NAPI bindings for QuickJS [`Context`].

use napi::bindgen_prelude::{Error, Result};
use napi_derive::napi;
use rquickjs::{Context as QuickContext, Runtime as QuickRuntime};

/// NAPI wrapper around a QuickJS execution context.
#[napi]
pub struct Context {
  pub(crate) inner: QuickContext,
}

#[napi]
impl Context {
  /// Create a new full QuickJS context for a runtime.
  #[napi(constructor)]
  pub fn new() -> Result<Self> {
    let rt = QuickRuntime::new().map_err(|e| Error::from_reason(e.to_string()))?;
    let ctx = QuickContext::full(&rt).map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(Self { inner: ctx })
  }

  /// Evaluate a JavaScript script string in this context and return the JSON result string.
  #[napi]
  pub fn eval(&self, script: String) -> Result<String> {
    self.inner.with(|ctx| {
      let val: rquickjs::Value = ctx.eval(script.as_str()).map_err(|e| Error::from_reason(e.to_string()))?;
      let json = ctx
        .json_stringify(val)
        .map_err(|e| Error::from_reason(e.to_string()))?
        .and_then(|s| s.to_string().ok())
        .unwrap_or_else(|| "null".to_string());
      Ok(json)
    })
  }

  /// Run garbage collection in this context.
  #[napi]
  pub fn run_gc(&self) {
    self.inner.with(|ctx| {
      ctx.run_gc();
    });
  }
}

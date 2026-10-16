use napi::bindgen_prelude::*;
use napi_derive::napi;
use rquickjs::{Context as QuickContext, Runtime as QuickRuntime};

/// NAPI wrapper around `rquickjs::Context`.
#[napi(js_name = "Context")]
pub struct JsContext {
  // Option used so we can manage lifetime or creation if needed, or direct owned Context
  pub(crate) inner: Option<QuickContext>,
}

#[napi]
impl JsContext {
  /// Create a new full QuickJS context given a QuickJS runtime pointer or standalone initialization.
  #[napi(factory)]
  pub fn full() -> Result<Self> {
    let rt = QuickRuntime::new().map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
    let ctx = QuickContext::full(&rt).map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
    Ok(Self { inner: Some(ctx) })
  }

  /// Create a base QuickJS context with only base intrinsics.
  #[napi(factory)]
  pub fn base() -> Result<Self> {
    let rt = QuickRuntime::new().map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
    let ctx = QuickContext::base(&rt).map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
    Ok(Self { inner: Some(ctx) })
  }

  /// Evaluate a JavaScript expression inside this QuickJS context.
  #[napi]
  pub fn eval(&self, code: String) -> Result<String> {
    let ctx = self.inner.as_ref().ok_or_else(|| {
      Error::new(Status::GenericFailure, "Context is uninitialized or disposed")
    })?;

    ctx.with(|ctx| {
      let res: String = ctx
        .eval(code)
        .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
      Ok(res)
    })
  }
}

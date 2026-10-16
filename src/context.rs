//! NAPI bindings for rquickjs `Context`.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rquickjs::Context as QuickContext;

use crate::runtime::QuickRuntime;

/// JS wrapper for rquickjs `Context`.
#[napi]
pub struct QuickContextWrapper {
  pub(crate) inner: QuickContext,
}

#[napi]
impl QuickContextWrapper {
  /// Create a base context with required functions registered.
  #[napi(factory)]
  pub fn base(runtime: &QuickRuntime) -> Result<Self> {
    let ctx = QuickContext::base(&runtime.inner)
      .map_err(|e| Error::from_reason(format!("Failed to create base Context: {e}")))?;
    Ok(Self { inner: ctx })
  }

  /// Create a full context with all standard intrinsics registered.
  #[napi(factory)]
  pub fn full(runtime: &QuickRuntime) -> Result<Self> {
    let ctx = QuickContext::full(&runtime.inner)
      .map_err(|e| Error::from_reason(format!("Failed to create full Context: {e}")))?;
    Ok(Self { inner: ctx })
  }

  /// Evaluate a JavaScript script string within this context.
  #[napi]
  pub fn eval(&self, script: String) -> Result<String> {
    self.inner.with(|ctx| {
      ctx
        .eval::<String, _>(script.as_str())
        .map_err(|e| Error::from_reason(format!("Evaluation error: {e}")))
    })
  }

  /// Evaluate a JavaScript script with eval options.
  #[napi]
  pub fn eval_with_options(&self, script: String, global: Option<bool>, strict: Option<bool>) -> Result<String> {
    let mut opts = rquickjs::context::EvalOptions::default();
    if let Some(g) = global {
      opts.global = g;
    }
    if let Some(s) = strict {
      opts.strict = s;
    }

    self.inner.with(|ctx| {
      ctx
        .eval_with_options::<String, _>(script.as_str(), opts)
        .map_err(|e| Error::from_reason(format!("Evaluation error: {e}")))
    })
  }
}

use napi_derive::napi;
use rquickjs::Context as QjsContext;

use crate::runtime::Runtime;

/// NAPI wrapper around `rquickjs::Context`.
#[napi]
pub struct Context {
  pub(crate) inner: QjsContext,
}

#[napi]
impl Context {
  /// Create a base context with only the required functions registered.
  #[napi(factory)]
  pub fn base(runtime: &Runtime) -> napi::Result<Self> {
    let ctx = QjsContext::base(&runtime.inner)
      .map_err(|e| napi::Error::from_reason(format!("Failed to create base context: {e}")))?;
    Ok(Self { inner: ctx })
  }

  /// Create a context with all standard available intrinsics registered.
  #[napi(factory)]
  pub fn full(runtime: &Runtime) -> napi::Result<Self> {
    let ctx = QjsContext::full(&runtime.inner)
      .map_err(|e| napi::Error::from_reason(format!("Failed to create full context: {e}")))?;
    Ok(Self { inner: ctx })
  }

  /// Evaluate JavaScript code within this context and return stringified result or error.
  #[napi]
  pub fn eval(&self, source: String) -> napi::Result<String> {
    self.inner.with(|ctx| {
      let val: rquickjs::Value = ctx
        .eval(source)
        .map_err(|e| napi::Error::from_reason(format!("Evaluation error: {e}")))?;

      if val.is_undefined() {
        Ok("undefined".to_string())
      } else if val.is_null() {
        Ok("null".to_string())
      } else if let Some(s) = val.as_string() {
        s.to_string()
          .map_err(|e| napi::Error::from_reason(format!("String conversion error: {e}")))
      } else {
        Ok(format!("{:?}", val))
      }
    })
  }
}

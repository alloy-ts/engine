//! NAPI bindings for QuickJS `Context` (`rquickjs::Context`).

use napi::bindgen_prelude::*;
use napi_derive::napi;
pub use rquickjs::Context as QjsContext;
use rquickjs::{Context as QContext, Runtime as QjsRuntime};

/// NAPI class exposing QuickJS `Context`.
#[napi]
pub struct QuickJSContext {
  _runtime: QjsRuntime,
  context: QContext,
}

#[napi]
impl QuickJSContext {
  /// Creates a full QuickJS execution context with standard intrinsics.
  #[napi(constructor)]
  pub fn new() -> Result<Self> {
    let runtime = QjsRuntime::new().map_err(|e| Error::from_reason(e.to_string()))?;
    let context = QContext::full(&runtime).map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(Self {
      _runtime: runtime,
      context,
    })
  }

  /// Evaluates JavaScript code within the context and returns the result string.
  #[napi]
  pub fn eval(&self, code: String) -> Result<String> {
    self.context.with(|ctx| {
      let val: rquickjs::Value = ctx.eval(code).map_err(|e| Error::from_reason(e.to_string()))?;
      if val.is_undefined() {
        Ok("undefined".to_string())
      } else if val.is_null() {
        Ok("null".to_string())
      } else if let Some(s) = val.as_string() {
        s.to_string().map_err(|e| Error::from_reason(e.to_string()))
      } else {
        let json = ctx.json_stringify(val).map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(json.map(|s| s.to_string().unwrap_or_default()).unwrap_or_else(|| "undefined".to_string()))
      }
    })
  }

  /// Evaluates JavaScript code and parses JSON output.
  #[napi(js_name = "eval_json")]
  pub fn eval_json(&self, code: String) -> Result<String> {
    self.context.with(|ctx| {
      let val: rquickjs::Value = ctx.eval(code).map_err(|e| Error::from_reason(e.to_string()))?;
      let json = ctx.json_stringify(val).map_err(|e| Error::from_reason(e.to_string()))?;
      Ok(json.map(|s| s.to_string().unwrap_or_default()).unwrap_or_else(|| "null".to_string()))
    })
  }
}

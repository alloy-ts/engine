//! NAPI bindings for `rquickjs::Context`.

use napi_derive::napi;
use rquickjs::Context as QjsContext;

use crate::engine::JsEngine as Engine;

/// JS-visible wrapper for `rquickjs::Context`.
#[napi(js_name = "Context")]
pub struct Context {
  pub(crate) inner: QjsContext,
}

#[napi]
impl Context {
  /// Create a new QuickJS Context from an optional Engine.
  ///
  /// If no Engine is provided, a new default QuickJS Runtime is created internally.
  #[napi(constructor)]
  pub fn new(engine: Option<&Engine>) -> napi::Result<Self> {
    let inner = match engine {
      Some(eng) => QjsContext::full(&eng.qjs_runtime)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?,
      None => {
        let rt = rquickjs::Runtime::new()
          .map_err(|e| napi::Error::from_reason(e.to_string()))?;
        QjsContext::full(&rt)
          .map_err(|e| napi::Error::from_reason(e.to_string()))?
      }
    };
    Ok(Self { inner })
  }

  /// Create a base Context with only required intrinsics registered.
  #[napi]
  pub fn base(engine: &Engine) -> napi::Result<Self> {
    let inner = QjsContext::base(&engine.qjs_runtime)
      .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    Ok(Self { inner })
  }

  /// Create a full Context with all standard intrinsics registered.
  #[napi]
  pub fn full(engine: &Engine) -> napi::Result<Self> {
    let inner = QjsContext::full(&engine.qjs_runtime)
      .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    Ok(Self { inner })
  }

  /// Evaluate a JavaScript script in this context and return the result as a JSON string.
  #[napi]
  pub fn eval(&self, script: String) -> napi::Result<String> {
    self.inner.with(|ctx| {
      let val: rquickjs::Value = ctx
        .eval(script.as_bytes())
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;

      if let Some(json_str) = ctx
        .json_stringify(&val)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?
      {
        json_str
          .to_string()
          .map_err(|e| napi::Error::from_reason(e.to_string()))
      } else {
        Ok("undefined".to_string())
      }
    })
  }

  /// Run garbage collection cycle for this context.
  #[napi]
  pub fn run_gc(&self) {
    self.inner.with(|ctx| {
      ctx.run_gc();
    });
  }
}

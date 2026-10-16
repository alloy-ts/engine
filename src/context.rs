//! NAPI bindings for QuickJS Context.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use crate::engine::Engine;

/// A single execution context with its own global variables and stack.
#[napi(js_name = "Context")]
pub struct Context {
  pub(crate) inner: rquickjs::Context,
}

#[napi]
impl Context {
  /// Create a base context.
  #[napi(factory, js_name = "base")]
  pub fn base(engine: &Engine) -> Result<Self> {
    let rt = engine.qjs_runtime.as_ref().ok_or_else(|| Error::from_reason("No QuickJS runtime available"))?;
    let ctx = rquickjs::Context::base(rt)
      .map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(Context { inner: ctx })
  }

  /// Create a full context with all standard intrinsics registered.
  #[napi(factory, js_name = "full")]
  pub fn full(engine: &Engine) -> Result<Self> {
    let rt = engine.qjs_runtime.as_ref().ok_or_else(|| Error::from_reason("No QuickJS runtime available"))?;
    let ctx = rquickjs::Context::full(rt)
      .map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(Context { inner: ctx })
  }

  /// Evaluate JavaScript source code in this context.
  #[napi(js_name = "eval")]
  pub fn eval(&self, source: String) -> Result<String> {
    self.inner.with(|ctx| {
      let val_res: std::result::Result<rquickjs::Value, _> = ctx.eval(source.as_bytes());
      match val_res {
        Ok(val) => {
          let json = ctx.json_stringify(val)
            .map_err(|e| Error::from_reason(e.to_string()))?;
          Ok(json.map(|s| s.to_string().unwrap_or_default()).unwrap_or_else(|| "undefined".to_string()))
        }
        Err(_e) => {
          let exception = ctx.catch();
          let json = ctx.json_stringify(exception).ok().flatten();
          let msg = json.map(|s| s.to_string().unwrap_or_default()).unwrap_or_else(|| "Error".to_string());
          Err(Error::from_reason(msg))
        }
      }
    })
  }
}

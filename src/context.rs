use napi::bindgen_prelude::*;
use napi_derive::napi;
use rquickjs::{Context as QuickContext, Value as QuickValue};

use crate::runtime::Runtime;

/// NAPI wrapper around `rquickjs::Context`.
///
/// A single execution context with its own global variables and stack.
#[napi]
pub struct Context {
  pub(crate) inner: QuickContext,
}

#[napi]
impl Context {
  /// Creates a context with all standard available intrinsics registered.
  #[napi(factory)]
  pub fn full(runtime: &Runtime) -> Result<Self> {
    let inner = QuickContext::full(&runtime.inner)
      .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to create full context: {e}")))?;
    Ok(Self { inner })
  }

  /// Creates a base context with only the required functions registered.
  #[napi(factory)]
  pub fn base(runtime: &Runtime) -> Result<Self> {
    let inner = QuickContext::base(&runtime.inner)
      .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to create base context: {e}")))?;
    Ok(Self { inner })
  }

  /// Evaluate JavaScript code within this context and return the result as a JSON string or primitive.
  #[napi]
  pub fn eval(&self, code: String) -> Result<String> {
    self.inner.with(|ctx| {
      let val: QuickValue = ctx
        .eval(code)
        .map_err(|e| Error::new(Status::GenericFailure, format!("Eval error: {e}")))?;

      if val.is_undefined() {
        Ok("undefined".to_string())
      } else if val.is_null() {
        Ok("null".to_string())
      } else if let Ok(json_opt) = ctx.json_stringify(&val) {
        if let Some(json_str) = json_opt {
          json_str
            .to_string()
            .map_err(|e| Error::new(Status::GenericFailure, format!("String error: {e}")))
        } else {
          Ok("undefined".to_string())
        }
      } else {
        Ok("[object Object]".to_string())
      }
    })
  }

  /// Executes a quickjs job if any are pending in this context.
  #[napi]
  pub fn execute_pending_job(&self) -> bool {
    self.inner.with(|ctx| ctx.execute_pending_job())
  }

  /// Runs the quickjs garbage collector for a cycle.
  #[napi]
  pub fn run_gc(&self) {
    self.inner.with(|ctx| ctx.run_gc());
  }
}

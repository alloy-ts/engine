//! QuickJS Engine NAPI bindings (`rquickjs::Runtime`).

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rquickjs::Runtime as QuickJSRuntime;

/// NAPI wrapper around QuickJS `Runtime`, named `Engine`.
#[napi]
pub struct Engine {
  pub(crate) inner: QuickJSRuntime,
}

#[napi]
impl Engine {
  /// Create a new QuickJS engine instance.
  #[napi(constructor)]
  pub fn new() -> Result<Self> {
    let rt = QuickJSRuntime::new().map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(Self { inner: rt })
  }

  /// Set maximum memory limit (in bytes) for the engine. 0 means unlimited.
  #[napi(js_name = "set_memory_limit")]
  pub fn set_memory_limit(&self, limit: u32) {
    self.inner.set_memory_limit(limit as usize);
  }

  /// Set maximum stack size (in bytes) for the engine.
  #[napi(js_name = "set_max_stack_size")]
  pub fn set_max_stack_size(&self, limit: u32) {
    self.inner.set_max_stack_size(limit as usize);
  }

  /// Set garbage collection memory threshold.
  #[napi(js_name = "set_gc_threshold")]
  pub fn set_gc_threshold(&self, threshold: u32) {
    self.inner.set_gc_threshold(threshold as usize);
  }

  /// Manually trigger garbage collection.
  #[napi(js_name = "run_gc")]
  pub fn run_gc(&self) {
    self.inner.run_gc();
  }

  /// Test if any async/job tasks are pending.
  #[napi(js_name = "is_job_pending")]
  pub fn is_job_pending(&self) -> bool {
    self.inner.is_job_pending()
  }

  /// Execute the first pending job from the runtime job queue.
  #[napi(js_name = "execute_pending_job")]
  pub fn execute_pending_job(&self) -> Result<bool> {
    self
      .inner
      .execute_pending_job()
      .map_err(|e| Error::from_reason(format!("{e:?}")))
  }
}

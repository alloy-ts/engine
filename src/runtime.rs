//! NAPI bindings for QuickJS Runtime.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rquickjs::Runtime as QuickRuntime;

/// NAPI JS wrapper around QuickJS Runtime.
#[napi]
pub struct Runtime {
  pub(crate) inner: QuickRuntime,
}

#[napi]
impl Runtime {
  /// Create a new QuickJS runtime instance.
  #[napi(constructor)]
  pub fn new() -> Result<Self> {
    let inner = QuickRuntime::new().map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(Runtime { inner })
  }

  /// Manually trigger garbage collection in the runtime.
  #[napi]
  pub fn run_gc(&self) {
    self.inner.run_gc();
  }

  /// Set the memory limit for this runtime in bytes.
  #[napi]
  pub fn set_memory_limit(&self, limit: u32) {
    self.inner.set_memory_limit(limit as usize);
  }

  /// Set the maximum stack size for this runtime in bytes.
  #[napi]
  pub fn set_max_stack_size(&self, limit: u32) {
    self.inner.set_max_stack_size(limit as usize);
  }

  /// Check if there are any pending asynchronous jobs.
  #[napi]
  pub fn is_job_pending(&self) -> bool {
    self.inner.is_job_pending()
  }

  /// Execute the first pending job if any exists.
  #[napi]
  pub fn execute_pending_job(&self) -> Result<bool> {
    self
      .inner
      .execute_pending_job()
      .map_err(|e| Error::from_reason(format!("Job execution error: {e:?}")))
  }
}

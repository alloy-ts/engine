//! NAPI bindings for QuickJS Runtime (`rquickjs::Runtime`).
//!
//! QuickJS runtime, entry point for managing execution contexts and memory thresholds.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rquickjs::Runtime as QjsRuntime;

#[napi]
pub struct Runtime {
  inner: QjsRuntime,
}

#[napi]
impl Runtime {
  /// Create a new QuickJS runtime.
  #[napi(constructor)]
  pub fn new() -> Result<Self> {
    let inner = QjsRuntime::new().map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(Self { inner })
  }

  /// Set the info string of the runtime.
  #[napi]
  pub fn set_info(&self, info: String) -> Result<()> {
    self.inner.set_info(info).map_err(|e| Error::from_reason(e.to_string()))
  }

  /// Set a limit on the max amount of memory the runtime will use (in bytes).
  #[napi]
  pub fn set_memory_limit(&self, limit: u32) {
    self.inner.set_memory_limit(limit as usize);
  }

  /// Set a limit on the max size of stack the runtime will use (in bytes).
  #[napi]
  pub fn set_max_stack_size(&self, limit: u32) {
    self.inner.set_max_stack_size(limit as usize);
  }

  /// Set a memory threshold for garbage collection.
  #[napi]
  pub fn set_gc_threshold(&self, threshold: u32) {
    self.inner.set_gc_threshold(threshold as usize);
  }

  /// Manually run the garbage collection.
  #[napi]
  pub fn run_gc(&self) {
    self.inner.run_gc();
  }

  /// Test for pending jobs. Returns true when at least one job is pending.
  #[napi]
  pub fn is_job_pending(&self) -> bool {
    self.inner.is_job_pending()
  }

  /// Execute first pending job. Returns true when job was executed or false when queue is empty.
  #[napi]
  pub fn execute_pending_job(&self) -> Result<bool> {
    self
      .inner
      .execute_pending_job()
      .map_err(|e| Error::from_reason(format!("{:?}", e)))
  }
}

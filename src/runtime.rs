//! NAPI bindings for rquickjs `Runtime`.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rquickjs::Runtime as QuickRuntimeInner;

/// JS wrapper around rquickjs `Runtime`.
#[napi]
pub struct QuickRuntime {
  pub(crate) inner: QuickRuntimeInner,
}

#[napi]
impl QuickRuntime {
  #[napi(constructor)]
  pub fn new() -> Result<Self> {
    let inner = QuickRuntimeInner::new()
      .map_err(|e| Error::from_reason(format!("Failed to create QuickJS Runtime: {e}")))?;
    Ok(Self { inner })
  }

  /// Set a limit on the maximum amount of memory the runtime will use (in bytes).
  #[napi]
  pub fn set_memory_limit(&self, limit: u32) {
    self.inner.set_memory_limit(limit as usize);
  }

  /// Set a limit on the maximum stack size the runtime will use (in bytes).
  #[napi]
  pub fn set_max_stack_size(&self, limit: u32) {
    self.inner.set_max_stack_size(limit as usize);
  }

  /// Set a memory threshold for garbage collection.
  #[napi]
  pub fn set_gc_threshold(&self, threshold: u32) {
    self.inner.set_gc_threshold(threshold as usize);
  }

  /// Manually run garbage collection cycle.
  #[napi]
  pub fn run_gc(&self) {
    self.inner.run_gc();
  }

  /// Test whether at least one job is pending.
  #[napi]
  pub fn is_job_pending(&self) -> bool {
    self.inner.is_job_pending()
  }

  /// Execute the first pending job in the queue.
  #[napi]
  pub fn execute_pending_job(&self) -> Result<bool> {
    self
      .inner
      .execute_pending_job()
      .map_err(|e| Error::from_reason(format!("Job execution error: {e}")))
  }
}

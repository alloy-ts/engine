//! NAPI bindings for QuickJS [`Runtime`].

use napi::bindgen_prelude::{Error, Result};
use napi_derive::napi;
use rquickjs::Runtime as QuickRuntime;

/// NAPI wrapper around a QuickJS runtime instance.
#[napi]
pub struct Runtime {
  pub(crate) inner: QuickRuntime,
}

#[napi]
impl Runtime {
  /// Create a new QuickJS runtime instance.
  #[napi(constructor)]
  pub fn new() -> Result<Self> {
    let rt = QuickRuntime::new().map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(Self { inner: rt })
  }

  /// Set maximum memory limit for the runtime in bytes (0 for unlimited).
  #[napi(js_name = "set_memory_limit")]
  pub fn set_memory_limit(&self, limit: u32) {
    self.inner.set_memory_limit(limit as usize);
  }

  /// Set maximum stack size for the runtime in bytes.
  #[napi(js_name = "set_max_stack_size")]
  pub fn set_max_stack_size(&self, limit: u32) {
    self.inner.set_max_stack_size(limit as usize);
  }

  /// Set GC threshold in bytes.
  #[napi(js_name = "set_gc_threshold")]
  pub fn set_gc_threshold(&self, threshold: u32) {
    self.inner.set_gc_threshold(threshold as usize);
  }

  /// Manually trigger garbage collection.
  #[napi(js_name = "run_gc")]
  pub fn run_gc(&self) {
    self.inner.run_gc();
  }

  /// Check if a job/promise tick is pending.
  #[napi(js_name = "is_job_pending")]
  pub fn is_job_pending(&self) -> bool {
    self.inner.is_job_pending()
  }

  /// Execute first pending job.
  #[napi(js_name = "execute_pending_job")]
  pub fn execute_pending_job(&self) -> Result<bool> {
    self
      .inner
      .execute_pending_job()
      .map_err(|e| Error::from_reason(e.to_string()))
  }
}

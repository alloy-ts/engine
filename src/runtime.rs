//! NAPI bindings for QuickJS Runtime (`rquickjs::Runtime`).

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rquickjs::Runtime as QuickJSRuntime;

/// NAPI wrapper around `rquickjs::Runtime`.
#[napi]
pub struct Runtime {
  pub(crate) inner: QuickJSRuntime,
}

#[napi]
impl Runtime {
  /// Create a new QuickJS runtime instance.
  #[napi(constructor)]
  pub fn new() -> Result<Self> {
    let rt = QuickJSRuntime::new().map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(Self { inner: rt })
  }

  /// Set maximum memory limit (in bytes) for the runtime. 0 means unlimited.
  #[napi]
  pub fn set_memory_limit(&self, limit: u32) {
    self.inner.set_memory_limit(limit as usize);
  }

  /// Set maximum stack size (in bytes) for the runtime.
  #[napi]
  pub fn set_max_stack_size(&self, limit: u32) {
    self.inner.set_max_stack_size(limit as usize);
  }

  /// Set garbage collection memory threshold.
  #[napi]
  pub fn set_gc_threshold(&self, threshold: u32) {
    self.inner.set_gc_threshold(threshold as usize);
  }

  /// Manually trigger garbage collection.
  #[napi]
  pub fn run_gc(&self) {
    self.inner.run_gc();
  }

  /// Test if any async/job tasks are pending.
  #[napi]
  pub fn is_job_pending(&self) -> bool {
    self.inner.is_job_pending()
  }

  /// Execute the first pending job from the runtime job queue.
  #[napi]
  pub fn execute_pending_job(&self) -> Result<bool> {
    self
      .inner
      .execute_pending_job()
      .map_err(|e| Error::from_reason(format!("{e:?}")))
  }
}

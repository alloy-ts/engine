//! NAPI bindings for QuickJS `Runtime` (`rquickjs::Runtime`).

use napi::bindgen_prelude::*;
use napi_derive::napi;
pub use rquickjs::Runtime as QjsRuntime;
use rquickjs::Runtime as QRuntime;

/// NAPI class exposing QuickJS `Runtime`.
#[napi]
pub struct QuickJSRuntime {
  runtime: QRuntime,
}

#[napi]
impl QuickJSRuntime {
  /// Creates a new QuickJS runtime instance.
  #[napi(constructor)]
  pub fn new() -> Result<Self> {
    let runtime = QRuntime::new().map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(Self { runtime })
  }

  /// Sets the maximum memory limit in bytes for the runtime.
  #[napi(js_name = "setMemoryLimit")]
  pub fn set_memory_limit(&self, limit: u32) {
    self.runtime.set_memory_limit(limit as usize);
  }

  /// Sets the maximum stack size in bytes for the runtime.
  #[napi(js_name = "setMaxStackSize")]
  pub fn set_max_stack_size(&self, limit: u32) {
    self.runtime.set_max_stack_size(limit as usize);
  }

  /// Sets the memory threshold in bytes for triggering garbage collection.
  #[napi(js_name = "setGcThreshold")]
  pub fn set_gc_threshold(&self, threshold: u32) {
    self.runtime.set_gc_threshold(threshold as usize);
  }

  /// Manually runs a cycle of cyclic garbage collection.
  #[napi(js_name = "runGc")]
  pub fn run_gc(&self) {
    self.runtime.run_gc();
  }

  /// Checks whether at least one job is pending in the runtime queue.
  #[napi(js_name = "isJobPending")]
  pub fn is_job_pending(&self) -> bool {
    self.runtime.is_job_pending()
  }

  /// Executes the next pending job in the runtime queue.
  #[napi(js_name = "executePendingJob")]
  pub fn execute_pending_job(&self) -> Result<bool> {
    self
      .runtime
      .execute_pending_job()
      .map_err(|e| Error::from_reason(e.to_string()))
  }
}

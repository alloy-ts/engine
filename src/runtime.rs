use napi::bindgen_prelude::*;
use napi_derive::napi;
use rquickjs::Runtime as QuickRuntime;

/// NAPI wrapper around `rquickjs::Runtime`.
#[napi(js_name = "Runtime")]
pub struct JsRuntime {
  pub(crate) inner: QuickRuntime,
}

#[napi]
impl JsRuntime {
  /// Create a new QuickJS runtime.
  #[napi(constructor)]
  pub fn new() -> Result<Self> {
    let rt = QuickRuntime::new().map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
    Ok(Self { inner: rt })
  }

  /// Manually run the garbage collection.
  #[napi]
  pub fn run_gc(&self) {
    self.inner.run_gc();
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

  /// Test for pending jobs in QuickJS queue.
  #[napi]
  pub fn is_job_pending(&self) -> bool {
    self.inner.is_job_pending()
  }

  /// Execute first pending job in QuickJS queue.
  #[napi]
  pub fn execute_pending_job(&self) -> Result<bool> {
    self
      .inner
      .execute_pending_job()
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
  }
}

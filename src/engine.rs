use napi::bindgen_prelude::*;
use napi_derive::napi;
use rquickjs::Runtime as QuickRuntime;

/// JS class wrapping the QuickJS runtime (`rquickjs::Runtime`).
///
/// JS: `const engine = new Engine();`
#[napi]
pub struct Engine {
  pub(crate) inner: QuickRuntime,
}

#[napi]
impl Engine {
  /// Create a new QuickJS runtime instance.
  #[napi(constructor)]
  pub fn new() -> Result<Self> {
    let inner = QuickRuntime::new()
      .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to create QuickJS runtime: {e}")))?;
    Ok(Self { inner })
  }

  /// Set the descriptive info of the runtime.
  #[napi]
  pub fn set_info(&self, info: String) -> Result<()> {
    self
      .inner
      .set_info(info)
      .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to set info: {e}")))
  }

  /// Set a limit on the max amount of memory (in bytes) the runtime will use.
  #[napi]
  pub fn set_memory_limit(&self, limit: u32) {
    self.inner.set_memory_limit(limit as usize);
  }

  /// Set a limit on the max size of stack (in bytes) the runtime will use.
  #[napi]
  pub fn set_max_stack_size(&self, limit: u32) {
    self.inner.set_max_stack_size(limit as usize);
  }

  /// Set a memory threshold (in bytes) for garbage collection.
  #[napi]
  pub fn set_gc_threshold(&self, threshold: u32) {
    self.inner.set_gc_threshold(threshold as usize);
  }

  /// Manually run garbage collection cycles.
  #[napi]
  pub fn run_gc(&self) {
    self.inner.run_gc();
  }

  /// Test for pending jobs in the queue.
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
      .map_err(|e| Error::new(Status::GenericFailure, format!("Job execution error: {e}")))
  }

  /// Alias for `execute_pending_job`.
  #[napi(js_name = "execPendingJob")]
  pub fn exec_pending_job(&self) -> Result<bool> {
    self.execute_pending_job()
  }
}

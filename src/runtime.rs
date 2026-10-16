use napi_derive::napi;
use rquickjs::Runtime as QjsRuntime;

/// NAPI wrapper around `rquickjs::Runtime`.
#[napi]
pub struct Runtime {
  pub(crate) inner: QjsRuntime,
}

#[napi]
impl Runtime {
  /// Create a new QuickJS runtime.
  #[napi(constructor)]
  pub fn new() -> napi::Result<Self> {
    let rt = QjsRuntime::new()
      .map_err(|e| napi::Error::from_reason(format!("Failed to create QuickJS runtime: {e}")))?;
    Ok(Self { inner: rt })
  }

  /// Set a limit on the maximum amount of memory (in bytes) the runtime will use.
  #[napi]
  pub fn set_memory_limit(&self, limit: u32) {
    self.inner.set_memory_limit(limit as usize);
  }

  /// Set a limit on the maximum stack size (in bytes) the runtime will use.
  #[napi]
  pub fn set_max_stack_size(&self, limit: u32) {
    self.inner.set_max_stack_size(limit as usize);
  }

  /// Set a memory threshold for garbage collection.
  #[napi]
  pub fn set_gc_threshold(&self, threshold: u32) {
    self.inner.set_gc_threshold(threshold as usize);
  }

  /// Manually trigger garbage collection.
  #[napi]
  pub fn run_gc(&self) {
    self.inner.run_gc();
  }

  /// Test if there are pending jobs in the job queue.
  #[napi]
  pub fn is_job_pending(&self) -> bool {
    self.inner.is_job_pending()
  }

  /// Execute the first pending job from the queue.
  #[napi]
  pub fn execute_pending_job(&self) -> napi::Result<bool> {
    self
      .inner
      .execute_pending_job()
      .map_err(|e| napi::Error::from_reason(format!("Error executing pending job: {e:?}")))
  }
}

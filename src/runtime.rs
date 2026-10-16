//! NAPI bindings for JS `Runtime`.

use napi_derive::napi;

/// QuickJS / JS runtime, entry point of execution.
#[napi]
pub struct Runtime {
  memory_limit: usize,
  max_stack_size: usize,
  gc_threshold: usize,
  info: String,
}

#[napi]
impl Runtime {
  /// Create a new runtime.
  #[napi(constructor)]
  pub fn new() -> Self {
    Runtime {
      memory_limit: 0,
      max_stack_size: 256 * 1024,
      gc_threshold: 0,
      info: String::new(),
    }
  }

  /// Set the info string of the runtime.
  #[napi]
  pub fn set_info(&mut self, info: String) {
    self.info = info;
  }

  /// Set a limit on the max amount of memory the runtime will use.
  #[napi]
  pub fn set_memory_limit(&mut self, limit: u32) {
    self.memory_limit = limit as usize;
  }

  /// Set a limit on the max size of stack the runtime will use.
  #[napi]
  pub fn set_max_stack_size(&mut self, limit: u32) {
    self.max_stack_size = limit as usize;
  }

  /// Set a memory threshold for garbage collection.
  #[napi]
  pub fn set_gc_threshold(&mut self, threshold: u32) {
    self.gc_threshold = threshold as usize;
  }

  /// Set debug flags for dumping memory.
  #[napi]
  pub fn set_dump_flags(&self, _flags: u32) {}

  /// Manually run garbage collection.
  #[napi]
  pub fn run_gc(&self) {}

  /// Test for pending jobs. Returns true when at least one job is pending.
  #[napi]
  pub fn is_job_pending(&self) -> bool {
    false
  }

  /// Execute first pending job.
  #[napi]
  pub fn execute_pending_job(&self) -> bool {
    false
  }
}

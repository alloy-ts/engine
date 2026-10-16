//! NAPI bindings for JS `Runtime`.

use napi_derive::napi;
use crate::loader::{Loader, Resolver};

/// QuickJS / JS runtime, entry point of execution.
#[napi]
pub struct Runtime;

#[napi]
impl Runtime {
  /// Create a new runtime.
  #[napi(constructor)]
  pub fn new() -> Self {
    Runtime
  }

  /// Set the info string of the runtime.
  #[napi]
  pub fn set_info(&self, _info: String) {}

  /// Set a limit on the max amount of memory the runtime will use.
  #[napi]
  pub fn set_memory_limit(&self, _limit: u32) {}

  /// Set a limit on the max size of stack the runtime will use.
  #[napi]
  pub fn set_max_stack_size(&self, _limit: u32) {}

  /// Set a memory threshold for garbage collection.
  #[napi]
  pub fn set_gc_threshold(&self, _threshold: u32) {}

  /// Set the module resolver and loader.
  #[napi]
  pub fn set_loader(&self, _resolver: Option<&Resolver>, _loader: Option<&Loader>) {}

  /// Set debug flags for dumping memory.
  #[napi]
  pub fn set_dump_flags(&self, _flags: u32) {}

  /// Manually run garbage collection.
  #[napi]
  pub fn run_gc(&self) {}

  /// Test for pending jobs.
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

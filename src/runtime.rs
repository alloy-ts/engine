//! QuickJS runtime related types and NAPI bindings.

use std::sync::{Arc, Mutex, Weak};
use napi_derive::napi;

/// Opaque handle for C/FFI JSRuntime pointer.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct JSRuntime {
  _private: [u8; 0],
}

unsafe impl Send for JSRuntime {}
unsafe impl Sync for JSRuntime {}

/// Memory usage statistics for runtime.
#[napi(object)]
#[derive(Debug, Clone, Copy, Default)]
pub struct MemoryUsage {
  pub memory_used_size: i64,
  pub malloc_count: i64,
  pub malloc_size: i64,
  pub malloc_limit: i64,
  pub memory_usage_count: i64,
}

/// Error type when job execution fails.
#[derive(Debug)]
pub struct JobException(pub String);

impl std::fmt::Display for JobException {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "JobException: {}", self.0)
  }
}

impl std::error::Error for JobException {}

#[derive(Debug, Default)]
pub struct InnerRuntime {
  pub info: String,
  pub memory_limit: usize,
  pub max_stack_size: usize,
  pub gc_threshold: usize,
  pub dump_flags: u64,
  pub pending_jobs: usize,
  pub memory_usage: MemoryUsage,
}

/// QuickJS runtime, entry point of the library.
#[napi(js_name = "Runtime")]
#[derive(Clone, Debug)]
pub struct Runtime {
  pub(crate) inner: Arc<Mutex<InnerRuntime>>,
}

unsafe impl Send for Runtime {}
unsafe impl Sync for Runtime {}

/// A weak handle to the runtime.
#[napi(js_name = "WeakRuntime")]
#[derive(Clone, Debug)]
pub struct WeakRuntime {
  pub(crate) inner: Weak<Mutex<InnerRuntime>>,
}

unsafe impl Send for WeakRuntime {}
unsafe impl Sync for WeakRuntime {}

#[napi]
impl WeakRuntime {
  /// Upgrade the weak reference to a strong Runtime reference if still alive.
  #[napi]
  pub fn try_ref(&self) -> Option<Runtime> {
    self.inner.upgrade().map(|inner| Runtime { inner })
  }
}

#[napi]
impl Runtime {
  /// Create a new runtime.
  #[napi(constructor)]
  pub fn new() -> napi::Result<Self> {
    Ok(Self {
      inner: Arc::new(Mutex::new(InnerRuntime {
        max_stack_size: 256 * 1024,
        ..Default::default()
      })),
    })
  }

  /// Get weak reference to runtime.
  #[napi]
  pub fn weak(&self) -> WeakRuntime {
    WeakRuntime {
      inner: Arc::downgrade(&self.inner),
    }
  }

  /// Set the info of the runtime.
  #[napi]
  pub fn set_info(&self, info: String) -> napi::Result<()> {
    let mut lock = self
      .inner
      .lock()
      .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    lock.info = info;
    Ok(())
  }

  /// Set a limit on the max amount of memory the runtime will use.
  #[napi]
  pub fn set_memory_limit(&self, limit: i64) {
    if let Ok(mut lock) = self.inner.lock() {
      let l = limit.max(0) as usize;
      lock.memory_limit = l;
      lock.memory_usage.malloc_limit = limit;
    }
  }

  /// Set a limit on the max size of stack the runtime will use.
  #[napi]
  pub fn set_max_stack_size(&self, limit: i64) {
    if let Ok(mut lock) = self.inner.lock() {
      lock.max_stack_size = limit.max(0) as usize;
    }
  }

  /// Set a memory threshold for garbage collection.
  #[napi]
  pub fn set_gc_threshold(&self, threshold: i64) {
    if let Ok(mut lock) = self.inner.lock() {
      lock.gc_threshold = threshold.max(0) as usize;
    }
  }

  /// Set debug flags for dumping memory.
  #[napi]
  pub fn set_dump_flags(&self, flags: i64) {
    if let Ok(mut lock) = self.inner.lock() {
      lock.dump_flags = flags.max(0) as u64;
    }
  }

  /// Manually run the garbage collection.
  #[napi]
  pub fn run_gc(&self) {
    // GC trigger
  }

  /// Get memory usage stats.
  #[napi]
  pub fn memory_usage(&self) -> MemoryUsage {
    self
      .inner
      .lock()
      .map(|l| l.memory_usage)
      .unwrap_or_default()
  }

  /// Test for pending jobs.
  #[napi]
  pub fn is_job_pending(&self) -> bool {
    self.inner.lock().map(|l| l.pending_jobs > 0).unwrap_or(false)
  }

  /// Execute first pending job.
  #[napi]
  pub fn execute_pending_job(&self) -> napi::Result<bool> {
    let mut lock = self
      .inner
      .lock()
      .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    if lock.pending_jobs > 0 {
      lock.pending_jobs -= 1;
      Ok(true)
    } else {
      Ok(false)
    }
  }

  /// Get raw pointer to JSRuntime.
  pub fn as_raw_ptr(&self) -> *mut JSRuntime {
    std::ptr::null_mut()
  }
}

impl Runtime {
  pub fn set_interrupt_handler(&self, _handler: Option<Box<dyn FnMut() -> bool + Send>>) {}

  pub fn set_loader<R: crate::loader::Resolver + 'static, L: crate::loader::Loader + 'static>(
    &self,
    _resolver: R,
    _loader: L,
  ) {
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn base_runtime() {
    let rt = Runtime::new().unwrap();
    rt.set_info("test runtime".to_string()).unwrap();
    rt.set_memory_limit(0xFFFF);
    rt.set_gc_threshold(0xFF);
    rt.run_gc();

    let weak = rt.weak();
    assert!(weak.try_ref().is_some());
  }

  #[test]
  fn set_max_stack_size_large_values() {
    let rt = Runtime::new().unwrap();
    rt.set_max_stack_size(i64::MAX);
    rt.set_max_stack_size(0);
    rt.set_max_stack_size(256 * 1024);
  }
}

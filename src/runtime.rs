//! QuickJS runtime related types and NAPI bindings.

use napi_derive::napi;

pub use rquickjs::qjs::JSRuntime;

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

/// QuickJS runtime, entry point of the library.
#[napi(js_name = "Runtime")]
#[derive(Clone)]
pub struct Runtime {
  pub(crate) inner: rquickjs::Runtime,
}

/// A weak handle to the runtime.
#[napi(js_name = "WeakRuntime")]
#[derive(Clone)]
pub struct WeakRuntime {
  pub(crate) inner: rquickjs::runtime::WeakRuntime,
}

#[napi]
impl WeakRuntime {
  /// Upgrade the weak reference to a strong Runtime reference if still alive.
  #[napi]
  pub fn try_ref(&self) -> Option<Runtime> {
    self.inner.try_ref().map(|inner| Runtime { inner })
  }
}

#[napi]
impl Runtime {
  /// Create a new runtime.
  #[napi(constructor)]
  pub fn new() -> napi::Result<Self> {
    let inner = rquickjs::Runtime::new().map_err(|e| napi::Error::from_reason(e.to_string()))?;
    Ok(Self { inner })
  }

  /// Get weak reference to runtime.
  #[napi]
  pub fn weak(&self) -> WeakRuntime {
    WeakRuntime {
      inner: self.inner.weak(),
    }
  }

  /// Set the info of the runtime.
  #[napi]
  pub fn set_info(&self, info: String) -> napi::Result<()> {
    self.inner.set_info(info).map_err(|e| napi::Error::from_reason(e.to_string()))
  }

  /// Set a limit on the max amount of memory the runtime will use.
  #[napi]
  pub fn set_memory_limit(&self, limit: i64) {
    self.inner.set_memory_limit(limit.max(0) as usize);
  }

  /// Set a limit on the max size of stack the runtime will use.
  #[napi]
  pub fn set_max_stack_size(&self, limit: i64) {
    self.inner.set_max_stack_size(limit.max(0) as usize);
  }

  /// Set a memory threshold for garbage collection.
  #[napi]
  pub fn set_gc_threshold(&self, threshold: i64) {
    self.inner.set_gc_threshold(threshold.max(0) as usize);
  }

  /// Set debug flags for dumping memory.
  #[napi]
  pub fn set_dump_flags(&self, flags: i64) {
    self.inner.set_dump_flags(flags.max(0) as u64);
  }

  /// Manually run the garbage collection.
  #[napi]
  pub fn run_gc(&self) {
    self.inner.run_gc();
  }

  /// Get memory usage stats.
  #[napi]
  pub fn memory_usage(&self) -> MemoryUsage {
    let usage = self.inner.memory_usage();
    MemoryUsage {
      memory_used_size: usage.memory_used_size as i64,
      malloc_count: usage.malloc_count as i64,
      malloc_size: usage.malloc_size as i64,
      malloc_limit: usage.malloc_limit as i64,
      memory_usage_count: usage.memory_used_count as i64,
    }
  }

  /// Test for pending jobs.
  #[napi]
  pub fn is_job_pending(&self) -> bool {
    self.inner.is_job_pending()
  }

  /// Execute first pending job.
  #[napi]
  pub fn execute_pending_job(&self) -> napi::Result<bool> {
    self
      .inner
      .execute_pending_job()
      .map_err(|e| napi::Error::from_reason(e.to_string()))
  }

  /// Set the module loader with built-in resolver and loader.
  #[napi(js_name = "setBuiltinLoader")]
  pub fn set_builtin_loader(&self) {
    self.inner.set_loader(
      rquickjs::loader::BuiltinResolver::default(),
      rquickjs::loader::BuiltinLoader::default(),
    );
  }

  /// Set the module loader with file resolver and script loader.
  #[napi(js_name = "setFileLoader")]
  pub fn set_file_loader(&self) {
    self.inner.set_loader(
      rquickjs::loader::FileResolver::default(),
      rquickjs::loader::ScriptLoader::default(),
    );
  }

  /// Set the module loader by kind names.
  #[napi(js_name = "setLoader")]
  pub fn set_loader_kinds(&self, resolver_type: Option<String>, loader_type: Option<String>) {
    let res_kind = resolver_type.as_deref().unwrap_or("file");
    let load_kind = loader_type.as_deref().unwrap_or("script");

    match (res_kind, load_kind) {
      ("builtin", "builtin") => {
        self.inner.set_loader(
          rquickjs::loader::BuiltinResolver::default(),
          rquickjs::loader::BuiltinLoader::default(),
        );
      }
      ("builtin", _) => {
        self.inner.set_loader(
          rquickjs::loader::BuiltinResolver::default(),
          rquickjs::loader::ScriptLoader::default(),
        );
      }
      _ => {
        self.inner.set_loader(
          rquickjs::loader::FileResolver::default(),
          rquickjs::loader::ScriptLoader::default(),
        );
      }
    }
  }

  pub fn as_raw_ptr(&self) -> *mut JSRuntime {
    std::ptr::null_mut()
  }
}

impl Runtime {
  pub fn set_interrupt_handler(&self, handler: Option<Box<dyn FnMut() -> bool + Send>>) {
    self.inner.set_interrupt_handler(handler.map(|h| h as Box<dyn FnMut() -> bool>));
  }

  pub fn set_loader<R, L>(&self, resolver: R, loader: L)
  where
    R: rquickjs::loader::Resolver + 'static,
    L: rquickjs::loader::Loader + 'static,
  {
    self.inner.set_loader(resolver, loader);
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

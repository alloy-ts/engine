//! NAPI bindings for QuickJS Runtime (`rquickjs::Runtime`).

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rquickjs::loader::BuiltinLoader;
use rquickjs::Runtime as QjsRuntime;

use crate::loader::{NativeBuiltinResolver, NativeFileResolver, NativeScriptLoader};

/// JS class wrapping the QuickJS Runtime (`rquickjs::Runtime`).
#[napi]
pub struct Runtime {
  pub(crate) inner: QjsRuntime,
}

#[napi]
impl Runtime {
  /// Create a new QuickJS runtime.
  #[napi(constructor)]
  pub fn new() -> Result<Self> {
    let inner = QjsRuntime::new().map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(Self { inner })
  }

  /// Manually run garbage collection.
  #[napi(js_name = "run_gc")]
  pub fn run_gc(&self) {
    self.inner.run_gc();
  }

  /// Set a limit on the maximum amount of memory the runtime will use (0 = unlimited).
  #[napi(js_name = "set_memory_limit")]
  pub fn set_memory_limit(&self, limit: u32) {
    self.inner.set_memory_limit(limit as usize);
  }

  /// Set a limit on the maximum stack size the runtime will use.
  #[napi(js_name = "set_max_stack_size")]
  pub fn set_max_stack_size(&self, limit: u32) {
    self.inner.set_max_stack_size(limit as usize);
  }

  /// Set a memory threshold for garbage collection.
  #[napi(js_name = "set_gc_threshold")]
  pub fn set_gc_threshold(&self, threshold: u32) {
    self.inner.set_gc_threshold(threshold as usize);
  }

  /// Set debug flags for dumping memory.
  #[napi(js_name = "set_dump_flags")]
  pub fn set_dump_flags(&self, flags: u32) {
    self.inner.set_dump_flags(flags as u64);
  }

  /// Set info description string for the runtime.
  #[napi(js_name = "set_info")]
  pub fn set_info(&self, info: String) -> Result<()> {
    self.inner.set_info(info).map_err(|e| Error::from_reason(e.to_string()))
  }

  /// Test whether any async job is pending in the runtime queue.
  #[napi(js_name = "is_job_pending")]
  pub fn is_job_pending(&self) -> bool {
    self.inner.is_job_pending()
  }

  /// Execute the first pending job in the queue. Returns true when a job was executed.
  #[napi(js_name = "execute_pending_job")]
  pub fn execute_pending_job(&self) -> Result<bool> {
    self.inner.execute_pending_job().map_err(|e| Error::from_reason(e.to_string()))
  }

  /// Configure default file resolver and script loader for module resolution.
  #[napi(js_name = "set_loader")]
  pub fn set_loader(&self, file_resolver: &mut NativeFileResolver, script_loader: &mut NativeScriptLoader) {
    if let (Some(r), Some(l)) = (file_resolver.inner.take(), script_loader.inner.take()) {
      self.inner.set_loader(r, l);
    }
  }

  /// Configure builtin module resolver and loader.
  #[napi(js_name = "set_builtin_loader")]
  pub fn set_builtin_loader(&self, builtin_resolver: &mut NativeBuiltinResolver) {
    if let Some(r) = builtin_resolver.inner.take() {
      self.inner.set_loader(r, BuiltinLoader::default());
    }
  }
}

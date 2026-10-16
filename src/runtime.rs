//! NAPI bindings for `rquickjs::Runtime`.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rquickjs::Runtime as QjsRuntime;

use crate::context::JsContext;
use crate::loader::{JsBuiltinLoader, JsBuiltinResolver, JsFileResolver, JsScriptLoader};

/// Memory usage statistics for QuickJS Runtime.
#[napi(object)]
pub struct JSMemoryUsage {
  pub malloc_size: i64,
  pub malloc_limit: i64,
  pub malloc_count: i64,
  pub memory_used_size: i64,
  pub atom_count: i64,
  pub atom_size: i64,
  pub str_count: i64,
  pub str_size: i64,
  pub obj_count: i64,
  pub obj_size: i64,
  pub prop_count: i64,
  pub prop_size: i64,
  pub shape_count: i64,
  pub shape_size: i64,
  pub js_func_count: i64,
  pub js_func_size: i64,
  pub js_func_code_size: i64,
  pub c_func_count: i64,
  pub array_count: i64,
  pub fast_array_count: i64,
  pub fast_array_elements: i64,
  pub binary_object_count: i64,
  pub binary_object_size: i64,
}

/// NAPI binding around [`rquickjs::Runtime`].
#[napi(js_name = "Runtime")]
pub struct JsRuntime {
  pub(crate) inner: QjsRuntime,
}

#[napi]
impl JsRuntime {
  /// Create a new QuickJS runtime instance.
  #[napi(constructor)]
  pub fn new() -> Result<Self> {
    let inner = QjsRuntime::new().map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(Self { inner })
  }

  /// Create a new [`JsContext`] using this runtime with standard intrinsics registered.
  #[napi(js_name = "create_context")]
  pub fn create_context(&self) -> Result<JsContext> {
    let inner = rquickjs::Context::full(&self.inner).map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(JsContext { inner })
  }

  /// Create a base [`JsContext`] using this runtime.
  #[napi(js_name = "create_base_context")]
  pub fn create_base_context(&self) -> Result<JsContext> {
    let inner = rquickjs::Context::base(&self.inner).map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(JsContext { inner })
  }

  /// Set the info description of the runtime.
  #[napi(js_name = "set_info")]
  pub fn set_info(&self, info: String) -> Result<()> {
    self.inner.set_info(info).map_err(|e| Error::from_reason(e.to_string()))
  }

  /// Set a limit on the maximum amount of memory the runtime will use (0 = unlimited).
  #[napi(js_name = "set_memory_limit")]
  pub fn set_memory_limit(&self, limit: i64) {
    self.inner.set_memory_limit(limit as usize);
  }

  /// Set a limit on the maximum stack size the runtime will use.
  #[napi(js_name = "set_max_stack_size")]
  pub fn set_max_stack_size(&self, limit: i64) {
    self.inner.set_max_stack_size(limit as usize);
  }

  /// Set a memory threshold for garbage collection.
  #[napi(js_name = "set_gc_threshold")]
  pub fn set_gc_threshold(&self, threshold: i64) {
    self.inner.set_gc_threshold(threshold as usize);
  }

  /// Set debug flags for dumping memory.
  #[napi(js_name = "set_dump_flags")]
  pub fn set_dump_flags(&self, flags: i64) {
    self.inner.set_dump_flags(flags as u64);
  }

  /// Manually run garbage collection.
  #[napi(js_name = "run_gc")]
  pub fn run_gc(&self) {
    self.inner.run_gc();
  }

  /// Test for pending jobs in the job queue.
  #[napi(js_name = "is_job_pending")]
  pub fn is_job_pending(&self) -> bool {
    self.inner.is_job_pending()
  }

  /// Execute the first pending job in the job queue.
  #[napi(js_name = "execute_pending_job")]
  pub fn execute_pending_job(&self) -> Result<bool> {
    match self.inner.execute_pending_job() {
      Ok(executed) => Ok(executed),
      Err(job_err) => Err(Error::from_reason(job_err.to_string())),
    }
  }

  /// Get memory usage statistics.
  #[napi(js_name = "memory_usage")]
  pub fn memory_usage(&self) -> JSMemoryUsage {
    let mem = self.inner.memory_usage();
    JSMemoryUsage {
      malloc_size: mem.malloc_size as i64,
      malloc_limit: mem.malloc_limit as i64,
      malloc_count: mem.malloc_count as i64,
      memory_used_size: mem.memory_used_size as i64,
      atom_count: mem.atom_count as i64,
      atom_size: mem.atom_size as i64,
      str_count: mem.str_count as i64,
      str_size: mem.str_size as i64,
      obj_count: mem.obj_count as i64,
      obj_size: mem.obj_size as i64,
      prop_count: mem.prop_count as i64,
      prop_size: mem.prop_size as i64,
      shape_count: mem.shape_count as i64,
      shape_size: mem.shape_size as i64,
      js_func_count: mem.js_func_count as i64,
      js_func_size: mem.js_func_size as i64,
      js_func_code_size: mem.js_func_code_size as i64,
      c_func_count: mem.c_func_count as i64,
      array_count: mem.array_count as i64,
      fast_array_count: mem.fast_array_count as i64,
      fast_array_elements: mem.fast_array_elements as i64,
      binary_object_count: mem.binary_object_count as i64,
      binary_object_size: mem.binary_object_size as i64,
    }
  }

  /// Configure standard file module loader/resolver for the runtime.
  #[napi(js_name = "set_file_loader")]
  pub fn set_file_loader(&self, resolver: &mut JsFileResolver, loader: &mut JsScriptLoader) {
    let res = resolver.inner.take().unwrap_or_default();
    let ldr = loader.inner.take().unwrap_or_default();
    self.inner.set_loader(res, ldr);
  }

  /// Configure builtin module loader/resolver for the runtime.
  #[napi(js_name = "set_builtin_loader")]
  pub fn set_builtin_loader(&self, resolver: &mut JsBuiltinResolver, loader: &mut JsBuiltinLoader) {
    let res = resolver.inner.take().unwrap_or_default();
    let ldr = loader.inner.take().unwrap_or_default();
    self.inner.set_loader(res, ldr);
  }
}

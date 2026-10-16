//! NAPI bindings for QuickJS [`Runtime`].

use napi::bindgen_prelude::*;
use napi::Error;
use napi_derive::napi;

/// Memory usage metrics of the QuickJS runtime.
#[napi(object)]
#[derive(Debug, Clone)]
pub struct JsMemoryUsage {
  #[napi(js_name = "malloc_size")]
  pub malloc_size: i64,
  #[napi(js_name = "malloc_limit")]
  pub malloc_limit: i64,
  #[napi(js_name = "memory_used_size")]
  pub memory_used_size: i64,
  #[napi(js_name = "malloc_count")]
  pub malloc_count: i64,
  #[napi(js_name = "memory_used_count")]
  pub memory_used_count: i64,
  #[napi(js_name = "atom_count")]
  pub atom_count: i64,
  #[napi(js_name = "atom_size")]
  pub atom_size: i64,
  #[napi(js_name = "str_count")]
  pub str_count: i64,
  #[napi(js_name = "str_size")]
  pub str_size: i64,
  #[napi(js_name = "obj_count")]
  pub obj_count: i64,
  #[napi(js_name = "obj_size")]
  pub obj_size: i64,
  #[napi(js_name = "prop_count")]
  pub prop_count: i64,
  #[napi(js_name = "prop_size")]
  pub prop_size: i64,
  #[napi(js_name = "shape_count")]
  pub shape_count: i64,
  #[napi(js_name = "shape_size")]
  pub shape_size: i64,
  #[napi(js_name = "js_func_count")]
  pub js_func_count: i64,
  #[napi(js_name = "js_func_size")]
  pub js_func_size: i64,
  #[napi(js_name = "js_func_code_size")]
  pub js_func_code_size: i64,
  #[napi(js_name = "c_func_count")]
  pub c_func_count: i64,
  #[napi(js_name = "array_count")]
  pub array_count: i64,
  #[napi(js_name = "fast_array_count")]
  pub fast_array_count: i64,
  #[napi(js_name = "fast_array_elements")]
  pub fast_array_elements: i64,
}

/// NAPI wrapper around QuickJS [`Runtime`].
#[napi(js_name = "Runtime")]
#[derive(Clone)]
pub struct JsRuntime {
  pub(crate) inner: rquickjs::Runtime,
}

#[napi]
impl JsRuntime {
  /// Create a new runtime.
  #[napi(constructor)]
  pub fn new() -> Result<Self> {
    let rt = rquickjs::Runtime::new().map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(Self { inner: rt })
  }

  /// Set user info string on the runtime.
  #[napi(js_name = "set_info")]
  pub fn set_info(&self, info: String) -> Result<()> {
    self
      .inner
      .set_info(info)
      .map_err(|e| Error::from_reason(e.to_string()))
  }

  /// Set a limit on the maximum amount of memory the runtime will use.
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
  pub fn set_dump_flags(&self, flags: u32) {
    self.inner.set_dump_flags(flags as u64);
  }

  /// Trigger cyclic garbage collection.
  #[napi(js_name = "run_gc")]
  pub fn run_gc(&self) {
    self.inner.run_gc();
  }

  /// Test for pending jobs.
  #[napi(js_name = "is_job_pending")]
  pub fn is_job_pending(&self) -> bool {
    self.inner.is_job_pending()
  }

  /// Execute the first pending job in queue.
  #[napi(js_name = "execute_pending_job")]
  pub fn execute_pending_job(&self) -> Result<bool> {
    self
      .inner
      .execute_pending_job()
      .map_err(|e| Error::from_reason(e.to_string()))
  }

  /// Retrieve memory usage statistics.
  #[napi(js_name = "memory_usage")]
  pub fn memory_usage(&self) -> JsMemoryUsage {
    let usage = self.inner.memory_usage();
    JsMemoryUsage {
      malloc_size: usage.malloc_size as i64,
      malloc_limit: usage.malloc_limit as i64,
      memory_used_size: usage.memory_used_size as i64,
      malloc_count: usage.malloc_count as i64,
      memory_used_count: usage.memory_used_count as i64,
      atom_count: usage.atom_count as i64,
      atom_size: usage.atom_size as i64,
      str_count: usage.str_count as i64,
      str_size: usage.str_size as i64,
      obj_count: usage.obj_count as i64,
      obj_size: usage.obj_size as i64,
      prop_count: usage.prop_count as i64,
      prop_size: usage.prop_size as i64,
      shape_count: usage.shape_count as i64,
      shape_size: usage.shape_size as i64,
      js_func_count: usage.js_func_count as i64,
      js_func_size: usage.js_func_size as i64,
      js_func_code_size: usage.js_func_code_size as i64,
      c_func_count: usage.c_func_count as i64,
      array_count: usage.array_count as i64,
      fast_array_count: usage.fast_array_count as i64,
      fast_array_elements: usage.fast_array_elements as i64,
    }
  }

  /// Set default file loader on the runtime.
  #[napi(js_name = "set_file_loader")]
  pub fn set_file_loader(&self) {
    self
      .inner
      .set_loader(rquickjs::loader::FileResolver::default(), rquickjs::loader::ScriptLoader::default());
  }
}

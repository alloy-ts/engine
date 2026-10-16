//! `JsEngine`: NAPI wrapper combining N-API environment utilities and QuickJS Runtime capabilities.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use napi::bindgen_prelude::*;
use napi::Env;
use napi_derive::napi;
use rquickjs::Runtime as QjsRuntime;

use crate::context::JsContext;
use crate::loader::{JsBuiltinLoader, JsBuiltinResolver, JsFileResolver, JsScriptLoader};

/// Memory usage statistics for QuickJS Engine / Runtime.
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

/// JS class wrapping N-API context (`napi::Env`) and QuickJS runtime.
#[napi(js_name = "Engine")]
pub struct JsEngine {
  pub(crate) env: Env,
  pub(crate) disposed: Arc<AtomicBool>,
  pub(crate) runtime: QjsRuntime,
}

#[napi]
impl JsEngine {
  /// Create a new `Engine`. Captures the injected `napi::Env` and builds a QuickJS runtime instance.
  #[napi(constructor)]
  pub fn new(env: Env) -> Result<Self> {
    let disposed = Arc::new(AtomicBool::new(false));

    let hook_data = Arc::clone(&disposed);
    let _ = env.add_env_cleanup_hook(hook_data, |flag: Arc<AtomicBool>| {
      flag.store(true, Ordering::SeqCst);
    });

    let runtime = QjsRuntime::new().map_err(|err| Error::from_reason(err.to_string()))?;

    Ok(Self {
      env,
      disposed,
      runtime,
    })
  }

  /// Whether the owning addon (and thus this context) has been unloaded.
  #[napi(js_name = "is_disposed")]
  pub fn is_disposed(&self) -> bool {
    self.disposed.load(Ordering::SeqCst)
  }

  #[napi(js_name = "create_array")]
  pub fn create_array(&self, len: u32) -> Result<Array<'_>> {
    self.ensure_alive()?;
    let mut raw_val = std::ptr::null_mut();
    napi::check_status!(
      unsafe { napi::sys::napi_create_array_with_length(self.env.raw(), len as usize, &mut raw_val) },
      "Create array with length failed"
    )?;
    unsafe { Array::from_napi_value(self.env.raw(), raw_val) }
  }

  /// `Engine::create_string` — build a JS string.
  #[napi(js_name = "create_string")]
  pub fn create_string(&self, value: String) -> Result<Unknown<'_>> {
    self.ensure_alive()?;
    self.env.create_string(&value)?.into_unknown(&self.env)
  }

  /// `Engine::create_int32` — build a JS number from an `i32`.
  #[napi(js_name = "create_int32")]
  pub fn create_int32(&self, int: i32) -> Result<Unknown<'_>> {
    self.ensure_alive()?;
    self.env.create_int32(int)?.into_unknown(&self.env)
  }

  /// `Engine::create_double` — build a JS number from an `f64`.
  #[napi(js_name = "create_double")]
  pub fn create_double(&self, value: f64) -> Result<Unknown<'_>> {
    self.ensure_alive()?;
    self.env.create_double(value)?.into_unknown(&self.env)
  }

  /// Does the global object have this property?
  #[napi(js_name = "has_global")]
  pub fn has_global(&self, name: String) -> Result<bool> {
    self.ensure_alive()?;
    let global = self.env.get_global()?;
    global.has_named_property(&name)
  }

  /// `Engine::get_napi_version` — the N-API version of the host runtime.
  #[napi(js_name = "get_napi_version")]
  pub fn get_napi_version(&self) -> Result<u32> {
    self.ensure_alive()?;
    self.env.get_napi_version()
  }

  /// `Object::new` through the captured context.
  #[napi(js_name = "make_object")]
  pub fn make_object(&self) -> Result<Object<'_>> {
    self.ensure_alive()?;
    let mut obj = Object::new(&self.env)?;
    obj.set_named_property("ok", true)?;
    obj.set_named_property("engine", "napi")?;
    Ok(obj)
  }

  /// `Engine::set_instance_data` — persist VM-specific state on the Agent.
  #[napi(js_name = "set_instance_data")]
  pub fn set_instance_data(&self, value: String) -> Result<()> {
    self.ensure_alive()?;
    self.env.set_instance_data(value, (), |_ctx| {})
  }

  /// `Engine::get_instance_data` — read back the persisted VM-specific state.
  #[napi(js_name = "get_instance_data")]
  pub fn get_instance_data(&self) -> Result<Option<String>> {
    self.ensure_alive()?;
    Ok(self.env.get_instance_data::<String>()?.map(|s: &mut String| s.clone()))
  }

  // --- QuickJS Runtime methods merged into Engine ---

  /// Create a new [`JsContext`] using this engine's QuickJS runtime with standard intrinsics registered.
  #[napi(js_name = "create_context")]
  pub fn create_context(&self) -> Result<JsContext> {
    let inner = rquickjs::Context::full(&self.runtime).map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(JsContext { inner })
  }

  /// Create a base [`JsContext`] using this engine's QuickJS runtime.
  #[napi(js_name = "create_base_context")]
  pub fn create_base_context(&self) -> Result<JsContext> {
    let inner = rquickjs::Context::base(&self.runtime).map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(JsContext { inner })
  }

  /// Set the info description of the QuickJS runtime.
  #[napi(js_name = "set_info")]
  pub fn set_info(&self, info: String) -> Result<()> {
    self.runtime.set_info(info).map_err(|e| Error::from_reason(e.to_string()))
  }

  /// Set a limit on the maximum amount of memory the QuickJS runtime will use (0 = unlimited).
  #[napi(js_name = "set_memory_limit")]
  pub fn set_memory_limit(&self, limit: i64) {
    self.runtime.set_memory_limit(limit as usize);
  }

  /// Set a limit on the maximum stack size the QuickJS runtime will use.
  #[napi(js_name = "set_max_stack_size")]
  pub fn set_max_stack_size(&self, limit: i64) {
    self.runtime.set_max_stack_size(limit as usize);
  }

  /// Set a memory threshold for garbage collection.
  #[napi(js_name = "set_gc_threshold")]
  pub fn set_gc_threshold(&self, threshold: i64) {
    self.runtime.set_gc_threshold(threshold as usize);
  }

  /// Set debug flags for dumping memory.
  #[napi(js_name = "set_dump_flags")]
  pub fn set_dump_flags(&self, flags: i64) {
    self.runtime.set_dump_flags(flags as u64);
  }

  /// Manually run QuickJS garbage collection.
  #[napi(js_name = "run_gc")]
  pub fn run_gc(&self) {
    self.runtime.run_gc();
  }

  /// Test for pending jobs in the QuickJS job queue.
  #[napi(js_name = "is_job_pending")]
  pub fn is_job_pending(&self) -> bool {
    self.runtime.is_job_pending()
  }

  /// Execute the first pending job in the QuickJS job queue.
  #[napi(js_name = "execute_pending_job")]
  pub fn execute_pending_job(&self) -> Result<bool> {
    match self.runtime.execute_pending_job() {
      Ok(executed) => Ok(executed),
      Err(job_err) => Err(Error::from_reason(job_err.to_string())),
    }
  }

  /// Get QuickJS memory usage statistics.
  #[napi(js_name = "memory_usage")]
  pub fn memory_usage(&self) -> JSMemoryUsage {
    let mem = self.runtime.memory_usage();
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

  /// Configure standard file module loader/resolver for QuickJS.
  #[napi(js_name = "set_file_loader")]
  pub fn set_file_loader(&self, resolver: &mut JsFileResolver, loader: &mut JsScriptLoader) {
    let res = resolver.inner.take().unwrap_or_default();
    let ldr = loader.inner.take().unwrap_or_default();
    self.runtime.set_loader(res, ldr);
  }

  /// Configure builtin module loader/resolver for QuickJS.
  #[napi(js_name = "set_builtin_loader")]
  pub fn set_builtin_loader(&self, resolver: &mut JsBuiltinResolver, loader: &mut JsBuiltinLoader) {
    let res = resolver.inner.take().unwrap_or_default();
    let ldr = loader.inner.take().unwrap_or_default();
    self.runtime.set_loader(res, ldr);
  }

  /// Guard: refuse to use a context that belongs to an unloaded addon.
  fn ensure_alive(&self) -> Result<()> {
    if self.disposed.load(Ordering::SeqCst) {
      return Err(Error::new(
        Status::GenericFailure,
        "Engine: N-API context has been disposed (addon unloaded)",
      ));
    }
    Ok(())
  }
}

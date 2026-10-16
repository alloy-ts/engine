//! `JsEngine`: a JS-visible NAPI class that owns the N-API context handle (`napi::Env`) and QuickJS `Runtime`.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use napi::bindgen_prelude::*;
use napi::Env;
use napi_derive::napi;

use crate::runtime::JsMemoryUsage;

/// JS class wrapping the N-API context (`napi::Env`) and QuickJS `Runtime`.
#[napi(js_name = "Engine")]
#[derive(Clone)]
pub struct JsEngine {
  pub(crate) env: Env,
  pub(crate) disposed: Arc<AtomicBool>,
  pub(crate) qjs_runtime: rquickjs::Runtime,
}

#[napi]
impl JsEngine {
  /// Capture the `napi::Env` that the runtime injects and register a
  /// cleanup hook, and initialize QuickJS runtime.
  #[napi(constructor)]
  pub fn new(env: Env) -> Result<Self> {
    let disposed = Arc::new(AtomicBool::new(false));

    let hook_data = Arc::clone(&disposed);
    env.add_env_cleanup_hook(hook_data, |flag: Arc<AtomicBool>| {
      flag.store(true, Ordering::SeqCst);
    })?;

    let qjs_runtime = rquickjs::Runtime::new().map_err(|e| Error::from_reason(e.to_string()))?;

    Ok(Self {
      env,
      disposed,
      qjs_runtime,
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

  // QuickJS Runtime methods on Engine:

  /// Set user info string on the runtime.
  #[napi(js_name = "set_info")]
  pub fn set_info(&self, info: String) -> Result<()> {
    self.ensure_alive()?;
    self
      .qjs_runtime
      .set_info(info)
      .map_err(|e| Error::from_reason(e.to_string()))
  }

  /// Set a limit on the maximum amount of memory the runtime will use.
  #[napi(js_name = "set_memory_limit")]
  pub fn set_memory_limit(&self, limit: i64) -> Result<()> {
    self.ensure_alive()?;
    self.qjs_runtime.set_memory_limit(limit as usize);
    Ok(())
  }

  /// Set a limit on the maximum stack size the runtime will use.
  #[napi(js_name = "set_max_stack_size")]
  pub fn set_max_stack_size(&self, limit: i64) -> Result<()> {
    self.ensure_alive()?;
    self.qjs_runtime.set_max_stack_size(limit as usize);
    Ok(())
  }

  /// Set a memory threshold for garbage collection.
  #[napi(js_name = "set_gc_threshold")]
  pub fn set_gc_threshold(&self, threshold: i64) -> Result<()> {
    self.ensure_alive()?;
    self.qjs_runtime.set_gc_threshold(threshold as usize);
    Ok(())
  }

  /// Set debug flags for dumping memory.
  #[napi(js_name = "set_dump_flags")]
  pub fn set_dump_flags(&self, flags: u32) -> Result<()> {
    self.ensure_alive()?;
    self.qjs_runtime.set_dump_flags(flags as u64);
    Ok(())
  }

  /// Trigger cyclic garbage collection.
  #[napi(js_name = "run_gc")]
  pub fn run_gc(&self) -> Result<()> {
    self.ensure_alive()?;
    self.qjs_runtime.run_gc();
    Ok(())
  }

  /// Test for pending jobs.
  #[napi(js_name = "is_job_pending")]
  pub fn is_job_pending(&self) -> Result<bool> {
    self.ensure_alive()?;
    Ok(self.qjs_runtime.is_job_pending())
  }

  /// Execute the first pending job in queue.
  #[napi(js_name = "execute_pending_job")]
  pub fn execute_pending_job(&self) -> Result<bool> {
    self.ensure_alive()?;
    self
      .qjs_runtime
      .execute_pending_job()
      .map_err(|e| Error::from_reason(e.to_string()))
  }

  /// Retrieve memory usage statistics.
  #[napi(js_name = "memory_usage")]
  pub fn memory_usage(&self) -> Result<JsMemoryUsage> {
    self.ensure_alive()?;
    let usage = self.qjs_runtime.memory_usage();
    Ok(JsMemoryUsage {
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
    })
  }

  /// Set default file loader on the runtime.
  #[napi(js_name = "set_file_loader")]
  pub fn set_file_loader(&self) -> Result<()> {
    self.ensure_alive()?;
    self
      .qjs_runtime
      .set_loader(rquickjs::loader::FileResolver::default(), rquickjs::loader::ScriptLoader::default());
    Ok(())
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

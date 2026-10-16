//! `Engine`: JS-visible NAPI class wrapping N-API context (`napi::Env`) and QuickJS runtime (`rquickjs::Runtime`).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use napi::bindgen_prelude::*;
use napi::Env;
use napi_derive::napi;

/// JS class wrapping N-API context (`napi::Env`) and QuickJS engine runtime.
#[napi(js_name = "Engine")]
pub struct Engine {
  pub(crate) env: Env,
  pub(crate) disposed: Arc<AtomicBool>,
  pub(crate) qjs_runtime: Option<rquickjs::Runtime>,
}

#[napi]
impl Engine {
  /// Constructor: injected by NAPI with current environment handle.
  #[napi(constructor)]
  pub fn new(env: Env) -> Result<Self> {
    let disposed = Arc::new(AtomicBool::new(false));

    let hook_data = Arc::clone(&disposed);
    env.add_env_cleanup_hook(hook_data, |flag: Arc<AtomicBool>| {
      flag.store(true, Ordering::SeqCst);
    })?;

    let qjs_runtime = rquickjs::Runtime::new().ok();

    Ok(Self {
      env,
      disposed,
      qjs_runtime,
    })
  }

  /// Whether the context has been disposed/unloaded.
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

  #[napi(js_name = "create_string")]
  pub fn create_string(&self, value: String) -> Result<Unknown<'_>> {
    self.ensure_alive()?;
    self.env.create_string(&value)?.into_unknown(&self.env)
  }

  #[napi(js_name = "create_int32")]
  pub fn create_int32(&self, int: i32) -> Result<Unknown<'_>> {
    self.ensure_alive()?;
    self.env.create_int32(int)?.into_unknown(&self.env)
  }

  #[napi(js_name = "create_double")]
  pub fn create_double(&self, value: f64) -> Result<Unknown<'_>> {
    self.ensure_alive()?;
    self.env.create_double(value)?.into_unknown(&self.env)
  }

  #[napi(js_name = "has_global")]
  pub fn has_global(&self, name: String) -> Result<bool> {
    self.ensure_alive()?;
    let global = self.env.get_global()?;
    global.has_named_property(&name)
  }

  #[napi(js_name = "get_napi_version")]
  pub fn get_napi_version(&self) -> Result<u32> {
    self.ensure_alive()?;
    self.env.get_napi_version()
  }

  #[napi(js_name = "make_object")]
  pub fn make_object(&self) -> Result<Object<'_>> {
    self.ensure_alive()?;
    let mut obj = Object::new(&self.env)?;
    obj.set_named_property("ok", true)?;
    obj.set_named_property("engine", "napi")?;
    Ok(obj)
  }

  #[napi(js_name = "set_instance_data")]
  pub fn set_instance_data(&self, value: String) -> Result<()> {
    self.ensure_alive()?;
    self.env.set_instance_data(value, (), |_ctx| {})
  }

  #[napi(js_name = "get_instance_data")]
  pub fn get_instance_data(&self) -> Result<Option<String>> {
    self.ensure_alive()?;
    Ok(self.env.get_instance_data::<String>()?.map(|s: &mut String| s.clone()))
  }

  /// Run QuickJS garbage collection cycle.
  #[napi(js_name = "run_gc")]
  pub fn run_gc(&self) {
    if let Some(rt) = &self.qjs_runtime {
      rt.run_gc();
    }
  }

  /// Set memory limit for QuickJS runtime.
  #[napi(js_name = "set_memory_limit")]
  pub fn set_memory_limit(&self, limit: u32) {
    if let Some(rt) = &self.qjs_runtime {
      rt.set_memory_limit(limit as usize);
    }
  }

  /// Set max stack size for QuickJS runtime.
  #[napi(js_name = "set_max_stack_size")]
  pub fn set_max_stack_size(&self, limit: u32) {
    if let Some(rt) = &self.qjs_runtime {
      rt.set_max_stack_size(limit as usize);
    }
  }

  /// Test if QuickJS runtime has pending jobs.
  #[napi(js_name = "is_job_pending")]
  pub fn is_job_pending(&self) -> bool {
    if let Some(rt) = &self.qjs_runtime {
      rt.is_job_pending()
    } else {
      false
    }
  }

  /// Execute pending QuickJS job.
  #[napi(js_name = "execute_pending_job")]
  pub fn execute_pending_job(&self) -> Result<bool> {
    if let Some(rt) = &self.qjs_runtime {
      rt.execute_pending_job()
        .map_err(|e| Error::from_reason(e.to_string()))
    } else {
      Ok(false)
    }
  }

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

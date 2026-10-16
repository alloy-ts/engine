//! `Engine`: a JS-visible NAPI class that owns the N-API context handle and QuickJS runtime.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use napi::bindgen_prelude::*;
use napi::{sys, NodeVersion};
use napi_derive::napi;
use rquickjs::Runtime as QuickRuntimeInner;
use crate::global::JsGlobal;

/// JS class wrapping the N-API context (`napi::Env`) and QuickJS `Runtime`.
#[napi]
pub struct Engine {
  pub(crate) env: Env,
  pub(crate) disposed: Arc<AtomicBool>,
  pub(crate) runtime: QuickRuntimeInner,
}

impl From<Env> for Engine {
  fn from(env: Env) -> Self {
    let runtime = QuickRuntimeInner::new().expect("Failed to create QuickJS runtime");
    Engine {
      env,
      disposed: Arc::new(AtomicBool::new(false)),
      runtime,
    }
  }
}

impl From<sys::napi_env> for Engine {
  fn from(env: sys::napi_env) -> Self {
    Engine::from(Env::from_raw(env))
  }
}

#[napi]
impl Engine {
  #[napi(constructor)]
  pub fn new(env: Env) -> Result<Self> {
    let disposed = Arc::new(AtomicBool::new(false));

    let hook_data = Arc::clone(&disposed);
    env.add_env_cleanup_hook(hook_data, |flag: Arc<AtomicBool>| {
      flag.store(true, Ordering::SeqCst);
    })?;

    let runtime = QuickRuntimeInner::new()
      .map_err(|e| Error::from_reason(format!("Failed to create QuickJS Runtime: {e}")))?;

    Ok(Self {
      env,
      disposed,
      runtime,
    })
  }

  pub fn from_raw(raw_env: sys::napi_env) -> Self {
    Engine::from(raw_env)
  }

  pub fn raw(&self) -> sys::napi_env {
    self.env.raw()
  }

  #[napi(js_name = "is_disposed")]
  pub fn is_disposed(&self) -> bool {
    self.disposed.load(Ordering::SeqCst)
  }

  #[napi(js_name = "create_array")]
  pub fn create_array(&self, len: u32) -> Result<Array<'_>> {
    self.ensure_alive()?;
    let obj = self.env.create_array(len)?;
    unsafe { Array::from_napi_value(self.env.raw(), obj.raw()) }
  }

  #[napi(js_name = "create_string")]
  pub fn create_string(&self, value: String) -> Result<Unknown<'_>> {
    self.ensure_alive()?;
    let s = self.env.create_string(&value)?;
    s.into_unknown(&self.env)
  }

  #[napi(js_name = "create_int32")]
  pub fn create_int32(&self, int: i32) -> Result<Unknown<'_>> {
    self.ensure_alive()?;
    let num = self.env.create_int32(int)?;
    num.into_unknown(&self.env)
  }

  #[napi(js_name = "create_int64")]
  pub fn create_int64(&self, int: i64) -> Result<Unknown<'_>> {
    self.ensure_alive()?;
    let num = self.env.create_int64(int)?;
    num.into_unknown(&self.env)
  }

  #[napi(js_name = "create_uint32")]
  pub fn create_uint32(&self, number: u32) -> Result<Unknown<'_>> {
    self.ensure_alive()?;
    let num = self.env.create_uint32(number)?;
    num.into_unknown(&self.env)
  }

  #[napi(js_name = "create_double")]
  pub fn create_double(&self, value: f64) -> Result<Unknown<'_>> {
    self.ensure_alive()?;
    let num = self.env.create_double(value)?;
    num.into_unknown(&self.env)
  }

  pub fn get_global(&self) -> Result<JsGlobal<'static>> {
    let mut global = std::ptr::null_mut();
    check_status!(
      unsafe { sys::napi_get_global(self.env.raw(), &mut global) },
      "Get global object from Env failed"
    )?;
    unsafe { JsGlobal::from_napi_value(self.env.raw(), global) }
  }

  #[napi(js_name = "has_global")]
  pub fn has_global(&self, name: String) -> Result<bool> {
    self.ensure_alive()?;
    self.get_global()?.has(&name)
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
    Ok(self.env.get_instance_data::<String>()?.cloned())
  }

  pub fn throw_error(&self, msg: &str, code: Option<&str>) -> Result<()> {
    self.ensure_alive()?;
    self.env.throw_error(msg, code)
  }

  pub fn throw_type_error(&self, msg: &str, code: Option<&str>) -> Result<()> {
    self.ensure_alive()?;
    self.env.throw_type_error(msg, code)
  }

  pub fn throw_range_error(&self, msg: &str, code: Option<&str>) -> Result<()> {
    self.ensure_alive()?;
    self.env.throw_range_error(msg, code)
  }

  pub fn get_node_version(&self) -> Result<NodeVersion> {
    self.ensure_alive()?;
    self.env.get_node_version()
  }

  // --- QuickJS Runtime methods ---

  /// Set a limit on the maximum amount of memory the runtime will use (in bytes).
  #[napi(js_name = "set_memory_limit")]
  pub fn set_memory_limit(&self, limit: u32) {
    self.runtime.set_memory_limit(limit as usize);
  }

  /// Set a limit on the maximum stack size the runtime will use (in bytes).
  #[napi(js_name = "set_max_stack_size")]
  pub fn set_max_stack_size(&self, limit: u32) {
    self.runtime.set_max_stack_size(limit as usize);
  }

  /// Set a memory threshold for garbage collection.
  #[napi(js_name = "set_gc_threshold")]
  pub fn set_gc_threshold(&self, threshold: u32) {
    self.runtime.set_gc_threshold(threshold as usize);
  }

  /// Manually run garbage collection cycle.
  #[napi(js_name = "run_gc")]
  pub fn run_gc(&self) {
    self.runtime.run_gc();
  }

  /// Test whether at least one job is pending.
  #[napi(js_name = "is_job_pending")]
  pub fn is_job_pending(&self) -> bool {
    self.runtime.is_job_pending()
  }

  /// Execute the first pending job in the queue.
  #[napi(js_name = "execute_pending_job")]
  pub fn execute_pending_job(&self) -> Result<bool> {
    self
      .runtime
      .execute_pending_job()
      .map_err(|e| Error::from_reason(format!("Job execution error: {e}")))
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

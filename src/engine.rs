//! QuickJS Engine NAPI bindings (`rquickjs::Runtime` & `napi::Env`).

use napi::bindgen_prelude::*;
use napi::Env;
use napi_derive::napi;
use rquickjs::Runtime as QuickJSRuntime;

/// NAPI wrapper around QuickJS `Runtime` and NAPI `Env` operations.
#[napi]
pub struct Engine {
  pub(crate) inner: QuickJSRuntime,
}

#[napi]
impl Engine {
  /// Create a new QuickJS engine instance.
  #[napi(constructor)]
  pub fn new(_env: &Env) -> Result<Self> {
    let rt = QuickJSRuntime::new().map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(Self { inner: rt })
  }

  /// Whether the engine/addon context is disposed (always false while loaded).
  #[napi(js_name = "is_disposed")]
  pub fn is_disposed(&self) -> bool {
    false
  }

  /// Allocate a JS array of the given length.
  #[napi(js_name = "create_array")]
  pub fn create_array<'a>(&self, env: &'a Env, len: u32) -> Result<Unknown<'a>> {
    env.create_array(len)?.into_unknown(env)
  }

  /// Build a JS string.
  #[napi(js_name = "create_string")]
  pub fn create_string<'a>(&self, env: &'a Env, value: String) -> Result<Unknown<'a>> {
    env.create_string(&value)?.into_unknown(env)
  }

  /// Build a JS number from an i32.
  #[napi(js_name = "create_int32")]
  pub fn create_int32<'a>(&self, env: &'a Env, int: i32) -> Result<Unknown<'a>> {
    env.create_int32(int)?.into_unknown(env)
  }

  /// Build a JS number from an f64.
  #[napi(js_name = "create_double")]
  pub fn create_double<'a>(&self, env: &'a Env, value: f64) -> Result<Unknown<'a>> {
    env.create_double(value)?.into_unknown(env)
  }

  /// Check if the global object has a property with `name`.
  #[napi(js_name = "has_global")]
  pub fn has_global(&self, env: &Env, name: String) -> Result<bool> {
    let global = env.get_global()?;
    global.has_named_property(&name)
  }

  /// Get host N-API version.
  #[napi(js_name = "get_napi_version")]
  pub fn get_napi_version(&self, env: &Env) -> Result<u32> {
    env.get_napi_version()
  }

  /// Create a sample object via N-API context.
  #[napi(js_name = "make_object")]
  pub fn make_object<'a>(&self, env: &'a Env) -> Result<Object<'a>> {
    let mut obj = Object::new(env)?;
    obj.set_named_property("ok", true)?;
    obj.set_named_property("engine", "napi")?;
    Ok(obj)
  }

  /// Persist VM-specific state on the instance.
  #[napi(js_name = "set_instance_data")]
  pub fn set_instance_data(&self, env: &Env, value: String) -> Result<()> {
    let mut instance_obj = Object::new(env)?;
    instance_obj.set_named_property("data", value)?;
    let mut global = env.get_global()?;
    global.set_named_property("__alloy_instance_data__", instance_obj)
  }

  /// Read back persisted VM-specific state.
  #[napi(js_name = "get_instance_data")]
  pub fn get_instance_data(&self, env: &Env) -> Result<Option<String>> {
    let global = env.get_global()?;
    if global.has_named_property("__alloy_instance_data__")? {
      let instance_obj: Object = global.get_named_property("__alloy_instance_data__")?;
      let data: String = instance_obj.get_named_property("data")?;
      Ok(Some(data))
    } else {
      Ok(None)
    }
  }

  /// Set maximum memory limit (in bytes) for the QuickJS engine. 0 means unlimited.
  #[napi(js_name = "set_memory_limit")]
  pub fn set_memory_limit(&self, limit: u32) {
    self.inner.set_memory_limit(limit as usize);
  }

  /// Set maximum stack size (in bytes) for the QuickJS engine.
  #[napi(js_name = "set_max_stack_size")]
  pub fn set_max_stack_size(&self, limit: u32) {
    self.inner.set_max_stack_size(limit as usize);
  }

  /// Set garbage collection memory threshold for QuickJS engine.
  #[napi(js_name = "set_gc_threshold")]
  pub fn set_gc_threshold(&self, threshold: u32) {
    self.inner.set_gc_threshold(threshold as usize);
  }

  /// Manually trigger QuickJS garbage collection.
  #[napi(js_name = "run_gc")]
  pub fn run_gc(&self) {
    self.inner.run_gc();
  }

  /// Test if any QuickJS async/job tasks are pending.
  #[napi(js_name = "is_job_pending")]
  pub fn is_job_pending(&self) -> bool {
    self.inner.is_job_pending()
  }

  /// Execute the first pending job from the QuickJS engine job queue.
  #[napi(js_name = "execute_pending_job")]
  pub fn execute_pending_job(&self) -> Result<bool> {
    self
      .inner
      .execute_pending_job()
      .map_err(|e| Error::from_reason(format!("{e:?}")))
  }

  /// Alias for `execute_pending_job` matching camelCase (`execPendingJob`).
  #[napi(js_name = "execPendingJob")]
  pub fn exec_pending_job(&self) -> Result<bool> {
    self.execute_pending_job()
  }
}

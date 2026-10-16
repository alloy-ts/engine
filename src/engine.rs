use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use napi::bindgen_prelude::{Error, Result, Status};
use napi::{Engine as NapiEngine, JsGlobal, Object, Unknown};
use napi_derive::napi;
use rquickjs::Runtime as QuickRuntime;

/// JS class wrapping both the N-API context (`napi::Engine`) and QuickJS runtime (`rquickjs::Runtime`).
///
/// JS: `const engine = new Engine();`
#[napi]
pub struct Engine(NapiEngine, Arc<AtomicBool>, QuickRuntime);

#[napi]
impl Engine {
  /// Capture the `napi::Engine` injected by NAPI and initialize QuickJS `rquickjs::Runtime`.
  #[napi(constructor)]
  pub fn new(napi_engine: NapiEngine) -> Result<Self> {
    let disposed = Arc::new(AtomicBool::new(false));

    let hook_data = Arc::clone(&disposed);
    napi_engine.add_engine_cleanup_hook(hook_data, |flag: Arc<AtomicBool>| {
      flag.store(true, Ordering::SeqCst);
    })?;

    let qjs_rt = QuickRuntime::new()
      .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to create QuickJS runtime: {e}")))?;

    Ok(Self(napi_engine, disposed, qjs_rt))
  }

  // --- N-API Context API ---

  /// Whether the owning addon (and thus this context) has been unloaded.
  #[napi]
  pub fn is_disposed(&self) -> bool {
    self.1.load(Ordering::SeqCst)
  }

  #[napi]
  pub fn create_array(&self, len: u32) -> Result<Unknown> {
    self.ensure_alive()?;
    Ok(self.0.create_array(len)?.into_unknown())
  }

  /// `Engine::create_string` — build a JS string.
  #[napi]
  pub fn create_string(&self, value: String) -> Result<Unknown> {
    self.ensure_alive()?;
    Ok(self.0.create_string(&value)?.into_unknown())
  }

  /// `Engine::create_int32` — build a JS number from an `i32`.
  #[napi]
  pub fn create_int32(&self, int: i32) -> Result<Unknown> {
    self.ensure_alive()?;
    Ok(self.0.create_int32(int)?.into_unknown())
  }

  /// `Engine::create_double` — build a JS number from an `f64`.
  #[napi]
  pub fn create_double(&self, value: f64) -> Result<Unknown> {
    self.ensure_alive()?;
    Ok(self.0.create_double(value)?.into_unknown())
  }

  /// Resolve the JS global object.
  pub fn get_global(&self) -> Result<JsGlobal<'static>> {
    self.0.get_global()
  }

  /// Does the global object have this property?
  #[napi]
  pub fn has_global(&self, name: String) -> Result<bool> {
    self.ensure_alive()?;
    self.get_global()?.has_named_property(&name)
  }

  /// `Engine::get_napi_version` — the N-API version of the host runtime.
  #[napi]
  pub fn get_napi_version(&self) -> Result<u32> {
    self.ensure_alive()?;
    self.0.get_napi_version()
  }

  /// `Object::new` through the captured context.
  #[napi]
  pub fn make_object(&self) -> Result<Object> {
    self.ensure_alive()?;
    let mut obj = Object::new(&self.0)?;
    obj.set_named_property("ok", true)?;
    obj.set_named_property("engine", "napi")?;
    Ok(obj)
  }

  /// `Engine::set_instance_data` — persist VM-specific state on the Agent.
  #[napi]
  pub fn set_instance_data(&self, value: String) -> Result<()> {
    self.ensure_alive()?;
    self.0.set_instance_data(value, (), |_ctx| {})
  }

  /// `Engine::get_instance_data` — read back the persisted VM-specific state.
  #[napi]
  pub fn get_instance_data(&self) -> Result<Option<String>> {
    self.ensure_alive()?;
    Ok(self.0.get_instance_data::<String>()?.map(|s| s.clone()))
  }

  /// Guard: refuse to use a context that belongs to an unloaded addon.
  fn ensure_alive(&self) -> Result<()> {
    if self.1.load(Ordering::SeqCst) {
      return Err(Error::new(
        Status::GenericFailure,
        "Engine: N-API context has been disposed (addon unloaded)",
      ));
    }
    Ok(())
  }

  // --- QuickJS Runtime API ---

  /// Set info string for QuickJS runtime.
  #[napi]
  pub fn set_info(&self, info: String) -> Result<()> {
    self
      .2
      .set_info(info)
      .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to set info: {e}")))
  }

  /// Set memory limit (in bytes) for QuickJS runtime.
  #[napi]
  pub fn set_memory_limit(&self, limit: u32) {
    self.2.set_memory_limit(limit as usize);
  }

  /// Set max stack size (in bytes) for QuickJS runtime.
  #[napi]
  pub fn set_max_stack_size(&self, limit: u32) {
    self.2.set_max_stack_size(limit as usize);
  }

  /// Set GC threshold (in bytes) for QuickJS runtime.
  #[napi]
  pub fn set_gc_threshold(&self, threshold: u32) {
    self.2.set_gc_threshold(threshold as usize);
  }

  /// Run QuickJS garbage collector.
  #[napi]
  pub fn run_gc(&self) {
    self.2.run_gc();
  }

  /// Check if a job is pending in QuickJS queue.
  #[napi]
  pub fn is_job_pending(&self) -> bool {
    self.2.is_job_pending()
  }

  /// Execute pending job in QuickJS queue.
  #[napi]
  pub fn execute_pending_job(&self) -> Result<bool> {
    self
      .2
      .execute_pending_job()
      .map_err(|e| Error::new(Status::GenericFailure, format!("Job execution error: {e}")))
  }

  /// Alias for `execute_pending_job`.
  #[napi(js_name = "execPendingJob")]
  pub fn exec_pending_job(&self) -> Result<bool> {
    self.execute_pending_job()
  }

  pub(crate) fn qjs_runtime(&self) -> &QuickRuntime {
    &self.2
  }
}

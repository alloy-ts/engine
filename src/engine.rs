//! NAPI bindings for `Engine`.
//!
//! JS-visible wrapper around the N-API context (`napi::Engine`) and QuickJS [`rquickjs::Runtime`].

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use napi::bindgen_prelude::{Error, Result, Status};
use napi::sys;
use napi::{Array, Engine as NapiEngine, JsGlobal, Object, Unknown, Value, ValueType};
use napi_derive::napi;
use rquickjs::Runtime as QuickRuntime;

/// JS class wrapping the N-API context (`napi::Engine`) and QuickJS runtime.
///
/// JS: `const engine = new Engine();` — NAPI injects `NapiEngine` when called from Node-API.
#[napi]
pub struct Engine {
  napi_engine: NapiEngine,
  disposed: Arc<AtomicBool>,
  qjs_runtime: Option<QuickRuntime>,
}

#[napi]
impl Engine {
  /// Capture the `napi::Engine` that the runtime injects and register a
  /// cleanup hook so we learn when it stops being valid.
  #[napi(constructor)]
  pub fn new(engine: NapiEngine) -> Result<Self> {
    let disposed = Arc::new(AtomicBool::new(false));

    let qjs_runtime = QuickRuntime::new().ok();

    let hook_data = Arc::clone(&disposed);
    engine.add_engine_cleanup_hook(hook_data, |flag: Arc<AtomicBool>| {
      flag.store(true, Ordering::SeqCst);
    })?;

    Ok(Self {
      napi_engine: engine,
      disposed,
      qjs_runtime,
    })
  }

  /// Whether the owning addon (and thus this context) has been unloaded.
  #[napi]
  pub fn is_disposed(&self) -> bool {
    self.disposed.load(Ordering::SeqCst)
  }

  #[napi]
  pub fn create_array(&self, len: u32) -> Result<Array> {
    let eng = self.ensure_napi()?;
    Ok(Array::new(eng.raw(), len))
  }

  /// `Engine::create_string` — build a JS string.
  #[napi]
  pub fn create_string(&self, value: String) -> Result<Unknown> {
    let eng = self.ensure_napi()?;
    Ok(eng.create_string(&value)?.into_unknown())
  }

  /// `Engine::create_int32` — build a JS number from an `i32`.
  #[napi]
  pub fn create_int32(&self, int: i32) -> Result<Unknown> {
    let eng = self.ensure_napi()?;
    Ok(eng.create_int32(int)?.into_unknown())
  }

  /// `Engine::create_double` — build a JS number from an `f64`.
  #[napi]
  pub fn create_double(&self, value: f64) -> Result<Unknown> {
    let eng = self.ensure_napi()?;
    Ok(eng.create_double(value)?.into_unknown())
  }

  /// Resolve the JS global object (mirrors `napi::Engine::get_global`).
  pub fn get_global(&self) -> Result<JsGlobal<'static>> {
    let eng = self.ensure_napi()?;
    let mut global = std::ptr::null_mut();
    napi::check_status!(
      unsafe { sys::napi_get_global(eng.raw(), &mut global) },
      "Get global object from Env failed"
    )?;
    Ok(JsGlobal(
      Value {
        value: global,
        env: eng.raw(),
        value_type: ValueType::Object,
      },
      std::marker::PhantomData,
    ))
  }

  /// Does the global object have this property?
  #[napi]
  pub fn has_global(&self, name: String) -> Result<bool> {
    self.get_global()?.has_named_property(&name)
  }

  /// `Engine::get_napi_version` — the N-API version of the host runtime.
  #[napi]
  pub fn get_napi_version(&self) -> Result<u32> {
    let eng = self.ensure_napi()?;
    eng.get_napi_version()
  }

  /// `Object::new` through the captured context.
  #[napi]
  pub fn make_object(&self) -> Result<Object> {
    let eng = self.ensure_napi()?;
    let mut obj = Object::new(eng)?;
    obj.set_named_property("ok", true)?;
    obj.set_named_property("engine", "napi")?;
    Ok(obj)
  }

  /// `Engine::set_instance_data` — persist VM-specific state on the Agent.
  #[napi]
  pub fn set_instance_data(&self, value: String) -> Result<()> {
    let eng = self.ensure_napi()?;
    eng.set_instance_data(value, (), |_ctx| {})
  }

  /// `Engine::get_instance_data` — read back the persisted VM-specific state.
  #[napi]
  pub fn get_instance_data(&self) -> Result<Option<String>> {
    let eng = self.ensure_napi()?;
    Ok(eng.get_instance_data::<String>()?.map(|s| s.clone()))
  }

  // --- QuickJS Runtime Methods ---

  /// Manually trigger garbage collection in the QuickJS runtime.
  #[napi]
  pub fn run_gc(&self) {
    if let Some(ref qjs) = self.qjs_runtime {
      qjs.run_gc();
    }
  }

  /// Set the memory limit for the QuickJS runtime in bytes.
  #[napi]
  pub fn set_memory_limit(&self, limit: u32) {
    if let Some(ref qjs) = self.qjs_runtime {
      qjs.set_memory_limit(limit as usize);
    }
  }

  /// Set the maximum stack size for the QuickJS runtime in bytes.
  #[napi]
  pub fn set_max_stack_size(&self, limit: u32) {
    if let Some(ref qjs) = self.qjs_runtime {
      qjs.set_max_stack_size(limit as usize);
    }
  }

  /// Check if there are any pending asynchronous jobs in QuickJS engine.
  #[napi]
  pub fn is_job_pending(&self) -> bool {
    self
      .qjs_runtime
      .as_ref()
      .map(|qjs| qjs.is_job_pending())
      .unwrap_or(false)
  }

  /// Execute the first pending QuickJS job.
  #[napi(js_name = "execute_pending_job")]
  pub fn execute_pending_job(&self) -> Result<bool> {
    if let Some(ref qjs) = self.qjs_runtime {
      qjs
        .execute_pending_job()
        .map_err(|e| Error::from_reason(format!("Job execution error: {e:?}")))
    } else {
      Ok(false)
    }
  }

  /// Alias for `execute_pending_job`.
  #[napi(js_name = "execPendingJob")]
  pub fn exec_pending_job(&self) -> Result<bool> {
    self.execute_pending_job()
  }

  /// Helper: ensure NAPI context is present and alive.
  fn ensure_napi(&self) -> Result<&NapiEngine> {
    if self.disposed.load(Ordering::SeqCst) {
      return Err(Error::new(
        Status::GenericFailure,
        "Engine: N-API context has been disposed (addon unloaded)",
      ));
    }
    Ok(&self.napi_engine)
  }
}

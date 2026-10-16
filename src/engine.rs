//! `Engine`: JS-visible NAPI class that combines both the host N-API context handle (`napi::Env`)
//! and an embedded QuickJS runtime (`rquickjs::Runtime`).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use napi::bindgen_prelude::{Error, JsObjectValue, Object, Result, Status, Unknown};
use napi::{Env, JsGlobal, JsValue};
use napi_derive::napi;

/// JS class wrapping both the N-API context (`napi::Env`) and QuickJS runtime (`rquickjs::Runtime`).
///
/// JS: `const engine = new Engine();`
#[napi]
pub struct Engine {
  disposed: Arc<AtomicBool>,
  qjs_runtime: rquickjs::Runtime,
}

#[napi]
impl Engine {
  /// Capture the `napi::Env` injected by NAPI (if called from JS) and initialize the embedded QuickJS runtime.
  #[napi(constructor)]
  pub fn new(env: &Env) -> Result<Self> {
    let disposed = Arc::new(AtomicBool::new(false));
    let hook_data = Arc::clone(&disposed);
    env.add_env_cleanup_hook(hook_data, |flag: Arc<AtomicBool>| {
      flag.store(true, Ordering::SeqCst);
    })?;

    let qjs_runtime = rquickjs::Runtime::new()
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

    Ok(Self {
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
  pub fn create_array<'a>(&self, env: &'a Env, len: u32) -> Result<Unknown<'a>> {
    self.ensure_alive()?;
    Ok(env.create_array(len)?.to_unknown())
  }

  /// `Engine::create_string` — build a JS string.
  #[napi(js_name = "create_string")]
  pub fn create_string<'a>(&self, env: &'a Env, value: String) -> Result<Unknown<'a>> {
    self.ensure_alive()?;
    Ok(env.create_string(&value)?.to_unknown())
  }

  /// `Engine::create_int32` — build a JS number from an `i32`.
  #[napi(js_name = "create_int32")]
  pub fn create_int32<'a>(&self, env: &'a Env, int: i32) -> Result<Unknown<'a>> {
    self.ensure_alive()?;
    Ok(env.create_int32(int)?.to_unknown())
  }

  /// `Engine::create_double` — build a JS number from an `f64`.
  #[napi(js_name = "create_double")]
  pub fn create_double<'a>(&self, env: &'a Env, value: f64) -> Result<Unknown<'a>> {
    self.ensure_alive()?;
    Ok(env.create_double(value)?.to_unknown())
  }

  /// Resolve the JS global object (mirrors `napi::Env::get_global`).
  pub fn get_global<'a>(&self, env: &'a Env) -> Result<JsGlobal<'a>> {
    env.get_global()
  }

  /// Does the global object have this property?
  #[napi(js_name = "has_global")]
  pub fn has_global(&self, env: &Env, name: String) -> Result<bool> {
    self.ensure_alive()?;
    self.get_global(env)?.has_named_property(&name)
  }

  /// `Engine::get_napi_version` — the N-API version of the host runtime.
  #[napi(js_name = "get_napi_version")]
  pub fn get_napi_version(&self, env: &Env) -> Result<u32> {
    self.ensure_alive()?;
    env.get_napi_version()
  }

  /// `Object::new` through the captured context.
  #[napi(js_name = "make_object")]
  pub fn make_object<'a>(&self, env: &'a Env) -> Result<Object<'a>> {
    self.ensure_alive()?;
    let mut obj = Object::new(env)?;
    obj.set_named_property("ok", true)?;
    obj.set_named_property("engine", "napi")?;
    Ok(obj)
  }

  /// `Engine::set_instance_data` — persist VM-specific state on the Agent.
  #[napi(js_name = "set_instance_data")]
  pub fn set_instance_data(&self, env: &Env, value: String) -> Result<()> {
    self.ensure_alive()?;
    env.set_instance_data(value, (), |_ctx| {})
  }

  /// `Engine::get_instance_data` — read back the persisted VM-specific state.
  #[napi(js_name = "get_instance_data")]
  pub fn get_instance_data(&self, env: &Env) -> Result<Option<String>> {
    self.ensure_alive()?;
    Ok(env.get_instance_data::<String>()?.map(|s: &mut String| s.clone()))
  }

  /// Set the module loader and resolver for file-based scripts with search paths.
  #[napi(js_name = "set_file_loader")]
  pub fn set_file_loader(&self, paths: Option<Vec<String>>) {
    let mut resolver = rquickjs::loader::FileResolver::default();
    if let Some(p) = paths {
      for path in p {
        resolver.add_path(path);
      }
    }
    let loader = rquickjs::loader::ScriptLoader::default();
    self.qjs_runtime.set_loader(resolver, loader);
  }

  /// Set a limit on the max amount of memory the runtime will use (in bytes).
  #[napi(js_name = "set_memory_limit")]
  pub fn set_memory_limit(&self, limit: u32) {
    self.qjs_runtime.set_memory_limit(limit as usize);
  }

  /// Set a limit on the max size of stack the runtime will use (in bytes).
  #[napi(js_name = "set_max_stack_size")]
  pub fn set_max_stack_size(&self, limit: u32) {
    self.qjs_runtime.set_max_stack_size(limit as usize);
  }

  /// Set a memory threshold for garbage collection (in bytes).
  #[napi(js_name = "set_gc_threshold")]
  pub fn set_gc_threshold(&self, threshold: u32) {
    self.qjs_runtime.set_gc_threshold(threshold as usize);
  }

  /// Manually run the garbage collection cycle.
  #[napi(js_name = "run_gc")]
  pub fn run_gc(&self) {
    self.qjs_runtime.run_gc();
  }

  /// Test whether at least one pending job exists in the queue.
  #[napi(js_name = "is_job_pending")]
  pub fn is_job_pending(&self) -> bool {
    self.qjs_runtime.is_job_pending()
  }

  /// Execute the first pending job in the queue.
  #[napi(js_name = "execute_pending_job")]
  pub fn execute_pending_job(&self) -> Result<bool> {
    self
      .qjs_runtime
      .execute_pending_job()
      .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
  }

  /// Alias for `execute_pending_job` (camelCase for JS users).
  #[napi(js_name = "execPendingJob")]
  pub fn exec_pending_job_camel(&self) -> Result<bool> {
    self.execute_pending_job()
  }

  /// Alias for `execute_pending_job` (snake_case).
  #[napi(js_name = "exec_pending_job")]
  pub fn exec_pending_job(&self) -> Result<bool> {
    self.execute_pending_job()
  }

  /// Reference to the underlying [`rquickjs::Runtime`].
  pub fn inner(&self) -> &rquickjs::Runtime {
    &self.qjs_runtime
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

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use napi::bindgen_prelude::*;
use napi::sys;
use napi_derive::napi;
use rquickjs::loader::BuiltinLoader;
use rquickjs::Runtime as QjsRuntime;

use crate::loader::{NativeBuiltinResolver, NativeFileResolver, NativeScriptLoader};

/// JS class wrapping the N-API context (`Env`) and QuickJS Runtime (`QjsRuntime`).
#[napi]
pub struct Engine {
  pub(crate) env: Env,
  pub(crate) disposed: Arc<AtomicBool>,
  pub(crate) rt: QjsRuntime,
}

#[napi]
impl Engine {
  /// Capture the `Env` that the runtime injects and register a
  /// cleanup hook so we learn when it stops being valid, and initialize QuickJS Runtime.
  #[napi(constructor)]
  pub fn new(env: Env) -> Result<Self> {
    let disposed = Arc::new(AtomicBool::new(false));

    let hook_data = Arc::clone(&disposed);
    env.add_env_cleanup_hook(hook_data, |flag: Arc<AtomicBool>| {
      flag.store(true, Ordering::SeqCst);
    })?;

    let rt = QjsRuntime::new().map_err(|e| Error::from_reason(e.to_string()))?;

    Ok(Self { env, disposed, rt })
  }

  /// Whether the owning addon (and thus this context) has been unloaded.
  #[napi(js_name = "is_disposed")]
  pub fn is_disposed(&self) -> bool {
    self.disposed.load(Ordering::SeqCst)
  }

  #[napi(js_name = "create_array")]
  pub fn create_array(&self, len: u32) -> Result<Array<'_>> {
    self.ensure_alive()?;
    self.env.create_array(len)
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
    self.env.get_global()?.has_named_property(&name)
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
    let data_ptr = Box::into_raw(Box::new(value));
    unsafe extern "C" fn finalize_string(
      _env: sys::napi_env,
      data: *mut std::ffi::c_void,
      _hint: *mut std::ffi::c_void,
    ) {
      if !data.is_null() {
        drop(Box::from_raw(data as *mut String));
      }
    }
    let status = unsafe {
      sys::napi_set_instance_data(
        self.env.raw(),
        data_ptr as *mut _,
        Some(finalize_string),
        std::ptr::null_mut(),
      )
    };
    napi::check_status!(status, "Failed to set instance data")?;
    Ok(())
  }

  /// `Engine::get_instance_data` — read back the persisted VM-specific state.
  #[napi(js_name = "get_instance_data")]
  pub fn get_instance_data(&self) -> Result<Option<String>> {
    self.ensure_alive()?;
    let mut data_ptr = std::ptr::null_mut();
    let status = unsafe { sys::napi_get_instance_data(self.env.raw(), &mut data_ptr) };
    napi::check_status!(status, "Failed to get instance data")?;
    if data_ptr.is_null() {
      Ok(None)
    } else {
      let s = unsafe { &*(data_ptr as *const String) };
      Ok(Some(s.clone()))
    }
  }

  /// Manually run QuickJS garbage collection.
  #[napi(js_name = "run_gc")]
  pub fn run_gc(&self) {
    self.rt.run_gc();
  }

  /// Set a limit on the maximum amount of memory the runtime will use (0 = unlimited).
  #[napi(js_name = "set_memory_limit")]
  pub fn set_memory_limit(&self, limit: u32) {
    self.rt.set_memory_limit(limit as usize);
  }

  /// Set a limit on the maximum stack size the runtime will use.
  #[napi(js_name = "set_max_stack_size")]
  pub fn set_max_stack_size(&self, limit: u32) {
    self.rt.set_max_stack_size(limit as usize);
  }

  /// Set a memory threshold for garbage collection.
  #[napi(js_name = "set_gc_threshold")]
  pub fn set_gc_threshold(&self, threshold: u32) {
    self.rt.set_gc_threshold(threshold as usize);
  }

  /// Set debug flags for dumping memory.
  #[napi(js_name = "set_dump_flags")]
  pub fn set_dump_flags(&self, flags: u32) {
    self.rt.set_dump_flags(flags as u64);
  }

  /// Set info description string for the runtime.
  #[napi(js_name = "set_info")]
  pub fn set_info(&self, info: String) -> Result<()> {
    self.rt.set_info(info).map_err(|e| Error::from_reason(e.to_string()))
  }

  /// Test whether any async job is pending in the runtime queue.
  #[napi(js_name = "is_job_pending")]
  pub fn is_job_pending(&self) -> bool {
    self.rt.is_job_pending()
  }

  /// Execute the first pending job in the queue. Returns true when a job was executed.
  #[napi(js_name = "execute_pending_job")]
  pub fn execute_pending_job(&self) -> Result<bool> {
    self.rt.execute_pending_job().map_err(|e| Error::from_reason(e.to_string()))
  }

  /// Configure default file resolver and script loader for module resolution.
  #[napi(js_name = "set_loader")]
  pub fn set_loader(
    &self,
    file_resolver: &mut NativeFileResolver,
    script_loader: &mut NativeScriptLoader,
  ) {
    if let (Some(r), Some(l)) = (file_resolver.inner.take(), script_loader.inner.take()) {
      self.rt.set_loader(r, l);
    }
  }

  /// Configure builtin module resolver and loader.
  #[napi(js_name = "set_builtin_loader")]
  pub fn set_builtin_loader(&self, builtin_resolver: &mut NativeBuiltinResolver) {
    if let Some(r) = builtin_resolver.inner.take() {
      self.rt.set_loader(r, BuiltinLoader::default());
    }
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

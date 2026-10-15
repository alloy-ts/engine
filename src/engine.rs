//! `Engine`: a JS-visible NAPI class that owns the N-API context handle.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use napi::bindgen_prelude::{
  Error, JsObjectValue, Object, Result, Status, ToNapiValue, Unknown,
};
use napi::{Env, JsGlobal};
use napi_derive::napi;

/// JS class wrapping the N-API context (`napi::Env`).
#[napi]
pub struct Engine(Env, Arc<AtomicBool>);

#[napi]
impl Engine {
  /// Capture the `napi::Env` that the runtime injects and register a
  /// cleanup hook so we learn when it stops being valid.
  #[napi(constructor)]
  pub fn new(env: &Env) -> Result<Self> {
    let e = *env;
    let disposed = Arc::new(AtomicBool::new(false));

    let hook_data = Arc::clone(&disposed);
    e.add_env_cleanup_hook(hook_data, |flag: Arc<AtomicBool>| {
      flag.store(true, Ordering::SeqCst);
    })?;

    Ok(Self(e, disposed))
  }

  /// Whether the owning addon (and thus this context) has been unloaded.
  #[napi(js_name = "is_disposed")]
  pub fn is_disposed(&self) -> bool {
    self.1.load(Ordering::SeqCst)
  }

  /// `Engine::create_array` — build a JS array with the given length.
  #[napi(js_name = "create_array")]
  pub fn create_array(&self, len: u32) -> Result<Unknown<'_>> {
    self.ensure_alive()?;
    self.0.create_array(len)?.into_unknown(&self.0)
  }

  /// `Engine::create_string` — build a JS string.
  #[napi(js_name = "create_string")]
  pub fn create_string(&self, value: String) -> Result<Unknown<'_>> {
    self.ensure_alive()?;
    self.0.create_string(&value)?.into_unknown(&self.0)
  }

  /// `Engine::create_int32` — build a JS number from an `i32`.
  #[napi(js_name = "create_int32")]
  pub fn create_int32(&self, int: i32) -> Result<Unknown<'_>> {
    self.ensure_alive()?;
    self.0.create_int32(int)?.into_unknown(&self.0)
  }

  /// `Engine::create_int64` — build a JS number from an `i64`.
  #[napi(js_name = "create_int64")]
  pub fn create_int64(&self, int: i64) -> Result<Unknown<'_>> {
    self.ensure_alive()?;
    self.0.create_int64(int)?.into_unknown(&self.0)
  }

  /// `Engine::create_uint32` — build a JS number from a `u32`.
  #[napi(js_name = "create_uint32")]
  pub fn create_uint32(&self, number: u32) -> Result<Unknown<'_>> {
    self.ensure_alive()?;
    self.0.create_uint32(number)?.into_unknown(&self.0)
  }

  /// `Engine::create_double` — build a JS number from an `f64`.
  #[napi(js_name = "create_double")]
  pub fn create_double(&self, value: f64) -> Result<Unknown<'_>> {
    self.ensure_alive()?;
    self.0.create_double(value)?.into_unknown(&self.0)
  }

  /// Resolve the JS global object.
  pub fn get_global(&self) -> Result<JsGlobal<'static>> {
    self.0.get_global()
  }

  /// Does the global object have this property?
  #[napi(js_name = "has_global")]
  pub fn has_global(&self, name: String) -> Result<bool> {
    self.ensure_alive()?;
    let global = self.get_global()?;
    global.has_named_property(&name)
  }

  /// `Engine::get_napi_version` — the N-API version of the host runtime.
  #[napi(js_name = "get_napi_version")]
  pub fn get_napi_version(&self) -> Result<u32> {
    self.ensure_alive()?;
    self.0.get_napi_version()
  }

  /// `Object::new` through the captured context.
  #[napi(js_name = "make_object")]
  pub fn make_object(&self) -> Result<Object<'_>> {
    self.ensure_alive()?;
    let mut obj = Object::new(&self.0)?;
    obj.set_named_property("ok", true)?;
    obj.set_named_property("engine", "napi")?;
    Ok(obj)
  }

  /// Create a JS object via the captured context.
  #[napi(js_name = "create_object")]
  pub fn create_object(&self) -> Result<Object<'_>> {
    self.ensure_alive()?;
    Object::new(&self.0)
  }

  /// Create an empty JS array.
  #[napi(js_name = "create_empty_array")]
  pub fn create_empty_array(&self) -> Result<Unknown<'_>> {
    self.ensure_alive()?;
    self.0.create_array(0)?.into_unknown(&self.0)
  }

  /// `Engine::set_instance_data` — persist VM-specific state on the Agent.
  #[napi(js_name = "set_instance_data")]
  pub fn set_instance_data(&self, value: String) -> Result<()> {
    self.ensure_alive()?;
    self.0.set_instance_data(value, (), |_ctx| {})
  }

  /// `Engine::get_instance_data` — read back the persisted VM-specific state.
  #[napi(js_name = "get_instance_data")]
  pub fn get_instance_data(&self) -> Result<Option<String>> {
    self.ensure_alive()?;
    Ok(self.0.get_instance_data::<String>()?.cloned())
  }

  /// Throw a JS error with an optional error code.
  #[napi(js_name = "throw_error")]
  pub fn throw_error(&self, msg: String, code: Option<String>) -> Result<()> {
    self.ensure_alive()?;
    self.0.throw_error(&msg, code.as_deref())
  }

  /// Run a script string in the engine context.
  #[napi(js_name = "run_script")]
  pub fn run_script(&self, script: String) -> Result<Unknown<'_>> {
    self.ensure_alive()?;
    self.0.run_script::<_, Unknown>(&script)
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
}

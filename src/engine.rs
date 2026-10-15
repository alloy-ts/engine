#![allow(deprecated)]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use napi::bindgen_prelude::{
  Error, JsObjectValue, Object, Result, Status, ToNapiValue, Unknown,
};
use napi::{sys, Env};
use napi_derive::napi;

pub type Callback = unsafe extern "C" fn(sys::napi_env, sys::napi_callback_info) -> sys::napi_value;

pub(crate) static EMPTY_VEC: Vec<u8> = vec![];

/// `Engine` is used to represent a context that the underlying N-API implementation can use to persist VM-specific state.
#[napi]
pub struct Engine(Env, Arc<AtomicBool>);

impl From<Env> for Engine {
  fn from(engine: Env) -> Self {
    let disposed = Arc::new(AtomicBool::new(false));
    Engine(engine, disposed)
  }
}

// NAPI bindings for NAPI engine
#[napi]
impl Engine {
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

  #[napi(js_name = "from_raw")]
  pub fn from_raw(env: &Env) -> Result<Self> {
    Self::new(env)
  }

  /// Whether the context has been unloaded.
  #[napi(js_name = "is_disposed")]
  pub fn is_disposed(&self) -> bool {
    self.1.load(Ordering::SeqCst)
  }

  /// Create a new JavaScript number from a Rust `i32`
  #[napi(js_name = "create_int32")]
  pub fn create_int32(&self, int: i32) -> Result<Unknown<'_>> {
    self.ensure_alive()?;
    self.0.create_int32(int)?.into_unknown(&self.0)
  }

  /// Create a new JavaScript number from a Rust `i64`
  #[napi(js_name = "create_int64")]
  pub fn create_int64(&self, int: i64) -> Result<Unknown<'_>> {
    self.ensure_alive()?;
    self.0.create_int64(int)?.into_unknown(&self.0)
  }

  /// Create a new JavaScript number from a Rust `u32`
  #[napi(js_name = "create_uint32")]
  pub fn create_uint32(&self, number: u32) -> Result<Unknown<'_>> {
    self.ensure_alive()?;
    self.0.create_uint32(number)?.into_unknown(&self.0)
  }

  /// Create a new JavaScript number from a Rust `f64`
  #[napi(js_name = "create_double")]
  pub fn create_double(&self, double: f64) -> Result<Unknown<'_>> {
    self.ensure_alive()?;
    self.0.create_double(double)?.into_unknown(&self.0)
  }

  /// Create a new JavaScript string from a Rust string
  #[napi(js_name = "create_string")]
  pub fn create_string(&self, s: String) -> Result<Unknown<'_>> {
    self.ensure_alive()?;
    self.0.create_string(&s)?.into_unknown(&self.0)
  }

  /// Create a new JavaScript array with a specific length
  #[napi(js_name = "create_array")]
  pub fn create_array(&self, len: u32) -> Result<Unknown<'_>> {
    self.ensure_alive()?;
    self.0.create_array(len)?.into_unknown(&self.0)
  }

  /// Create an empty JS object
  #[napi(js_name = "create_object")]
  pub fn create_object(&self) -> Result<Object<'_>> {
    self.ensure_alive()?;
    Object::new(&self.0)
  }

  /// Create an empty JS array
  #[napi(js_name = "create_empty_array")]
  pub fn create_empty_array(&self) -> Result<Unknown<'_>> {
    self.ensure_alive()?;
    self.0.create_array(0)?.into_unknown(&self.0)
  }

  /// Check property existence on global
  #[napi(js_name = "has_global")]
  pub fn has_global(&self, name: String) -> Result<bool> {
    self.ensure_alive()?;
    let global = self.0.get_global()?;
    global.has_named_property(&name)
  }

  /// Get NAPI version
  #[napi(js_name = "get_napi_version")]
  pub fn get_napi_version(&self) -> Result<u32> {
    self.ensure_alive()?;
    self.0.get_napi_version()
  }

  /// Object::new through captured context
  #[napi(js_name = "make_object")]
  pub fn make_object(&self) -> Result<Object<'_>> {
    self.ensure_alive()?;
    let mut obj = Object::new(&self.0)?;
    obj.set_named_property("ok", true)?;
    obj.set_named_property("engine", "napi")?;
    Ok(obj)
  }

  /// Persist instance data
  #[napi(js_name = "set_instance_data")]
  pub fn set_instance_data(&self, value: String) -> Result<()> {
    self.ensure_alive()?;
    self.0.set_instance_data(value, (), |_ctx| {})
  }

  /// Read back instance data
  #[napi(js_name = "get_instance_data")]
  pub fn get_instance_data(&self) -> Result<Option<String>> {
    self.ensure_alive()?;
    Ok(self.0.get_instance_data::<String>()?.cloned())
  }

  /// Throw JavaScript Error
  #[napi(js_name = "throw_error")]
  pub fn throw_error(&self, msg: String, code: Option<String>) -> Result<()> {
    self.ensure_alive()?;
    self.0.throw_error(&msg, code.as_deref())
  }

  /// Throw JavaScript RangeError
  #[napi(js_name = "throw_range_error")]
  pub fn throw_range_error(&self, msg: String, code: Option<String>) -> Result<()> {
    self.ensure_alive()?;
    self.0.throw_range_error(&msg, code.as_deref())
  }

  /// Throw JavaScript TypeError
  #[napi(js_name = "throw_type_error")]
  pub fn throw_type_error(&self, msg: String, code: Option<String>) -> Result<()> {
    self.ensure_alive()?;
    self.0.throw_type_error(&msg, code.as_deref())
  }

  /// Run JavaScript script
  #[napi(js_name = "run_script")]
  pub fn run_script(&self, script: String) -> Result<Unknown<'_>> {
    self.ensure_alive()?;
    self.0.run_script::<_, Unknown>(&script)
  }

  /// Get raw sys napi_env pointer
  pub fn raw(&self) -> sys::napi_env {
    self.0.raw()
  }

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

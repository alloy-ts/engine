//! `Engine`: a JS-visible NAPI class that owns the N-API context handle.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use napi::bindgen_prelude::*;
use napi::{sys, NodeVersion};
use napi_derive::napi;
use crate::global::JsGlobal;

/// JS class wrapping the N-API context (`napi::Env`).
#[napi]
pub struct Engine(pub(crate) Env, pub(crate) Arc<AtomicBool>);

impl From<Env> for Engine {
  fn from(env: Env) -> Self {
    Engine(env, Arc::new(AtomicBool::new(false)))
  }
}

impl From<sys::napi_env> for Engine {
  fn from(env: sys::napi_env) -> Self {
    Engine(Env::from_raw(env), Arc::new(AtomicBool::new(false)))
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

    Ok(Self(env, disposed))
  }

  pub fn from_raw(raw_env: sys::napi_env) -> Self {
    Engine::from(raw_env)
  }

  pub fn raw(&self) -> sys::napi_env {
    self.0.raw()
  }

  #[napi(js_name = "is_disposed")]
  pub fn is_disposed(&self) -> bool {
    self.1.load(Ordering::SeqCst)
  }

  #[napi(js_name = "create_array")]
  pub fn create_array(&self, len: u32) -> Result<Array<'_>> {
    self.ensure_alive()?;
    let obj = self.0.create_array(len)?;
    unsafe { Array::from_napi_value(self.0.raw(), obj.raw()) }
  }

  #[napi(js_name = "create_string")]
  pub fn create_string(&self, value: String) -> Result<Unknown<'_>> {
    self.ensure_alive()?;
    let s = self.0.create_string(&value)?;
    s.into_unknown(&self.0)
  }

  #[napi(js_name = "create_int32")]
  pub fn create_int32(&self, int: i32) -> Result<Unknown<'_>> {
    self.ensure_alive()?;
    let num = self.0.create_int32(int)?;
    num.into_unknown(&self.0)
  }

  #[napi(js_name = "create_int64")]
  pub fn create_int64(&self, int: i64) -> Result<Unknown<'_>> {
    self.ensure_alive()?;
    let num = self.0.create_int64(int)?;
    num.into_unknown(&self.0)
  }

  #[napi(js_name = "create_uint32")]
  pub fn create_uint32(&self, number: u32) -> Result<Unknown<'_>> {
    self.ensure_alive()?;
    let num = self.0.create_uint32(number)?;
    num.into_unknown(&self.0)
  }

  #[napi(js_name = "create_double")]
  pub fn create_double(&self, value: f64) -> Result<Unknown<'_>> {
    self.ensure_alive()?;
    let num = self.0.create_double(value)?;
    num.into_unknown(&self.0)
  }

  pub fn get_global(&self) -> Result<JsGlobal<'static>> {
    let mut global = std::ptr::null_mut();
    check_status!(
      unsafe { sys::napi_get_global(self.0.raw(), &mut global) },
      "Get global object from Env failed"
    )?;
    unsafe { JsGlobal::from_napi_value(self.0.raw(), global) }
  }

  #[napi(js_name = "has_global")]
  pub fn has_global(&self, name: String) -> Result<bool> {
    self.ensure_alive()?;
    self.get_global()?.has(&name)
  }

  #[napi(js_name = "get_napi_version")]
  pub fn get_napi_version(&self) -> Result<u32> {
    self.ensure_alive()?;
    self.0.get_napi_version()
  }

  #[napi(js_name = "make_object")]
  pub fn make_object(&self) -> Result<Object<'_>> {
    self.ensure_alive()?;
    let mut obj = Object::new(&self.0)?;
    obj.set_named_property("ok", true)?;
    obj.set_named_property("engine", "napi")?;
    Ok(obj)
  }

  #[napi(js_name = "set_instance_data")]
  pub fn set_instance_data(&self, value: String) -> Result<()> {
    self.ensure_alive()?;
    self.0.set_instance_data(value, (), |_ctx| {})
  }

  #[napi(js_name = "get_instance_data")]
  pub fn get_instance_data(&self) -> Result<Option<String>> {
    self.ensure_alive()?;
    Ok(self.0.get_instance_data::<String>()?.cloned())
  }

  pub fn throw_error(&self, msg: &str, code: Option<&str>) -> Result<()> {
    self.ensure_alive()?;
    self.0.throw_error(msg, code)
  }

  pub fn throw_type_error(&self, msg: &str, code: Option<&str>) -> Result<()> {
    self.ensure_alive()?;
    self.0.throw_type_error(msg, code)
  }

  pub fn throw_range_error(&self, msg: &str, code: Option<&str>) -> Result<()> {
    self.ensure_alive()?;
    self.0.throw_range_error(msg, code)
  }

  pub fn get_node_version(&self) -> Result<NodeVersion> {
    self.ensure_alive()?;
    self.0.get_node_version()
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

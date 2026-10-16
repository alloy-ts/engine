use napi::bindgen_prelude::*;
use napi::Env;
use napi_derive::napi;

/// `Engine`: JS-visible struct wrapping NAPI context.
#[napi]
pub struct Engine {
  #[allow(dead_code)]
  env: sys::napi_env,
}

#[napi]
impl Engine {
  #[napi(constructor)]
  pub fn new(env: &Env) -> Self {
    Self { env: env.raw() }
  }

  #[napi(js_name = "is_disposed")]
  pub fn is_disposed(&self) -> bool {
    false
  }

  #[napi(js_name = "create_array")]
  pub fn create_array<'a>(&self, env: &'a Env, len: u32) -> Result<Unknown<'a>> {
    env.create_array(len)?.into_unknown(env)
  }

  #[napi(js_name = "create_string")]
  pub fn create_string<'a>(&self, env: &'a Env, value: String) -> Result<Unknown<'a>> {
    env.create_string(&value)?.into_unknown(env)
  }

  #[napi(js_name = "create_int32")]
  pub fn create_int32<'a>(&self, env: &'a Env, int: i32) -> Result<Unknown<'a>> {
    env.create_int32(int)?.into_unknown(env)
  }

  #[napi(js_name = "create_double")]
  pub fn create_double<'a>(&self, env: &'a Env, value: f64) -> Result<Unknown<'a>> {
    env.create_double(value)?.into_unknown(env)
  }

  #[napi(js_name = "has_global")]
  pub fn has_global(&self, env: &Env, name: String) -> Result<bool> {
    let global = env.get_global()?;
    global.has_named_property(&name)
  }

  #[napi(js_name = "get_napi_version")]
  pub fn get_napi_version(&self, env: &Env) -> Result<u32> {
    env.get_napi_version()
  }

  #[napi(js_name = "make_object")]
  pub fn make_object<'a>(&self, env: &'a Env) -> Result<Object<'a>> {
    let mut obj = Object::new(env)?;
    obj.set_named_property("ok", true)?;
    obj.set_named_property("engine", "napi")?;
    Ok(obj)
  }

  #[napi(js_name = "set_instance_data")]
  pub fn set_instance_data(&self, env: &Env, value: String) -> Result<()> {
    let mut instance_obj = Object::new(env)?;
    instance_obj.set_named_property("data", value)?;
    let mut global = env.get_global()?;
    global.set_named_property("__alloy_instance_data__", instance_obj)
  }

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
}

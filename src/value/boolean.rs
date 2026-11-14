use napi::bindgen_prelude::{Env, FnArgs, Function, JsObjectValue, JsValue, Object as JsObject, Result};
use napi_derive::napi;

/// Boolean binding draft.
///
/// The native `Boolean` constructor is used for boxing. The constructor itself
/// is a function object and therefore belongs on `Function.prototype`; the
/// boxed Boolean instance remains on `Boolean.prototype` as in JavaScript.
#[napi(js_name = "Boolean")]
pub mod boolean_api {
  use super::*;

  #[napi]
  pub fn boxed(env: Env, value: bool) -> Result<JsObject<'static>> {
    let boolean: Function<'static, FnArgs<(bool,)>, JsObject<'static>> =
      env.get_global()?.get_named_property("Boolean")?;
    boolean.new_instance((value,).into())?.coerce_to_object()
  }

  #[napi]
  pub fn function_prototype(env: Env) -> Result<JsObject<'static>> {
    let function: JsObject<'static> = env.get_global()?.get_named_property("Function")?;
    function.get_named_property("prototype")
  }
}

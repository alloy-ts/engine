use napi::bindgen_prelude::{Env, JsObjectValue, Object, Object as JsObject, Result, Unknown};
use napi_derive::napi;

use super::{host, reflect};
use crate::value::{self, big_integer, big_num, float, integer, small_number};

/// JavaScript object helpers backed by the host runtime's built-in `Object`.
///
/// This is intentionally a NAPI namespace, not `#[napi(object)]`: object-shape
/// structs are cloned values and cannot expose methods or static constructors.
#[napi(js_name = "Object")]
pub mod object_api {
    use super::*;

    #[napi]
    pub fn create<'env>(env: &'env Env, parent: Option<JsObject>) -> Result<JsObject<'env>> {
        reflect::create(env, parent)
    }

    #[napi(js_name = "fromParent")]
    pub fn from_parent<'env>(env: &'env Env, parent: JsObject) -> Result<JsObject<'env>> {
        reflect::create(env, Some(parent))
    }

    #[napi]
    pub fn extend_from<'env>(env: &'env Env, parent: JsObject) -> Result<JsObject<'env>> {
        reflect::create(env, Some(parent))
    }

    #[napi]
    pub fn assign<'env>(
        env: &'env Env,
        target: JsObject,
        source: JsObject,
    ) -> Result<JsObject<'env>> {
        reflect::assign(env, target, source)
    }
}

/// Register the namespace without replacing `globalThis.Object`.
#[napi(module_exports)]
pub fn init(mut exports: Object, env: Env) -> Result<()> {
    host::install(&env)?;
    let (value, value_prototype) = value::register(&mut exports, &env)?;
    integer::register(&mut exports, &env, value.clone(), value_prototype.clone())?;
    big_integer::register(&mut exports, &env)?;
    float::register(&mut exports, &env, value.clone(), value_prototype)?;
    small_number::register(&mut exports, &env, value)?;
    big_num::register(&mut exports, &env)?;
    let mut object_api: JsObject = exports.get_named_property("Object")?;
    let extend_from: Unknown = object_api.get_named_property("extendFrom")?;
    object_api.set_named_property("extends", extend_from)?;
    env.get_global()?
        .set_named_property("CoreObject", object_api)?;
    Ok(())
}

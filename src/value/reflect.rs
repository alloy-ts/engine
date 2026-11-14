use napi::bindgen_prelude::{
    Env, FnArgs, Function, JsObjectValue, JsValue, Null, Object as JsObject, Result, ToNapiValue,
    Unknown,
};

use super::host;

/// Access the host runtime's built-in `Object` constructor.
pub(crate) fn builtin_object(env: &Env) -> Result<JsObject<'_>> {
    host::object(env)
}

pub(crate) fn create<'value>(
    env: &Env,
    parent: Option<JsObject<'value>>,
) -> Result<JsObject<'static>> {
    let object: JsObject<'static> = env
        .get_global()?
        .get_named_property::<Unknown>("Object")?
        .coerce_to_object()?;
    let create: Function<'static, FnArgs<(Unknown<'value>,)>, JsObject<'static>> =
        object.get_named_property("create")?;
    let parent = match parent {
        Some(value) => value.to_unknown(),
        None => Null.into_unknown(env)?,
    };
    create.call((parent,).into())
}

pub(crate) fn assign<'value>(
    env: &Env,
    target: JsObject<'value>,
    source: JsObject<'value>,
) -> Result<JsObject<'static>> {
    let object: JsObject<'static> = env
        .get_global()?
        .get_named_property::<Unknown>("Object")?
        .coerce_to_object()?;
    let assign: Function<'static, FnArgs<(JsObject<'value>, JsObject<'value>)>, JsObject<'static>> =
        object.get_named_property("assign")?;
    assign.call((target, source).into())
}

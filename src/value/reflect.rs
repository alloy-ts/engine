use napi::bindgen_prelude::{
    Env, FnArgs, Function, JsValue, Null, Object as JsObject, Result, ToNapiValue, Unknown,
};

use super::host;

/// `Object.create(parent)` through the function captured at addon
/// initialization, so later user patches of `Object` or `Object.create` cannot
/// intercept wrapper or prototype creation.
pub(crate) fn create<'env>(env: &'env Env, parent: Option<JsObject<'_>>) -> Result<JsObject<'env>> {
    let create: Function<FnArgs<(Unknown,)>, JsObject<'env>> =
        host::intrinsic(env, "Object.create")?;
    let parent = match parent {
        Some(value) => value.to_unknown(),
        None => Null.into_unknown(env)?,
    };
    create.call((parent,).into())
}

/// `Object.assign(target, source)` through the captured intrinsic.
pub(crate) fn assign<'env>(
    env: &'env Env,
    target: JsObject<'_>,
    source: JsObject<'_>,
) -> Result<JsObject<'env>> {
    let assign: Function<FnArgs<(JsObject, JsObject)>, JsObject<'env>> =
        host::intrinsic(env, "Object.assign")?;
    assign.call((target, source).into())
}
